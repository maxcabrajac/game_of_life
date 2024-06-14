use std::{io::Write, thread::sleep, time::Duration};

#[derive(Clone, Copy, Debug)]
enum Cell {
	Dead,
	Alive,
}

mod array {
    use std::ops::{Index, IndexMut};

	pub struct Array2d<T> {
		w: usize,
		h: usize,
		v: Vec<T>,
	}

	impl<T> Array2d<T> {
		pub fn new_with<F>(w: usize, h: usize, mut init: F) -> Self where F: FnMut(usize, usize) -> T {
			let mut v = Vec::with_capacity(w * h);
			for i in 0..h {
				for j in 0..w {
					v.push(init(i, j));
				}
			}

			Array2d {
				w,
				h,
				v,
			}
		}

		pub fn dims(&self) -> (usize, usize) {
			(self.w, self.h)
		}
	}

	impl<T> Index<(usize, usize)> for Array2d<T> {
		type Output = T;
		fn index(&self, index: (usize, usize)) -> &Self::Output {
			assert!(index.0 < self.h);
			assert!(index.1 < self.w);
			&self.v[index.0 * self.w + index.1]
		}
	}

	impl<T> IndexMut<(usize, usize)> for Array2d<T> {
		fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
			assert!(index.0 < self.h);
			assert!(index.1 < self.w);
			&mut self.v[index.0 * self.w + index.1]
		}
	}

	impl<T: Default> Array2d<T> {
		pub fn new_default(w: usize, h: usize) -> Self {
			Self::new_with(w, h, |_, _| T::default())
		}
	}

	impl<T: Clone> Array2d<T> {
		pub fn new(w: usize, h: usize, val: T) -> Self {
			Self::new_with(w, h, |_, _| val.clone())
		}
	}
}

use array::Array2d;

type Buff = Array2d<Cell>;

fn random_buff(w: usize, h: usize, alive_prob: f64) -> Buff {
	use rand::prelude::*;

	let mut rng = rand::thread_rng();
	let mut random_state = || {
		match rng.gen_bool(alive_prob) {
			true => Cell::Alive,
			false => Cell::Dead,
		}
	};

	Buff::new_with(w, h, |_i, _j| random_state())
}

struct GameOfLife {
	buffs: (Buff, Buff),
	epoch_parity: bool,
}

impl GameOfLife {
	fn cell_evolution(buff: &Buff, i: usize, j: usize) -> Cell {
		const NEIGHBORS_DIRS_X: [isize; 8] = [-1, 0, 1, -1, 1, -1, 0, 1];
		const NEIGHBORS_DIRS_Y: [isize; 8] = [-1, -1, -1, 0, 0, 1, 1, 1];

		let neighbors_dirs  = NEIGHBORS_DIRS_X.iter().zip(NEIGHBORS_DIRS_Y.iter());

		let (w, h) = buff.dims();

		let neighbors = neighbors_dirs.map(|(dx, dy)| {
			let move_wrapping = |x: usize, dx: isize, limit: usize| {
				let mut x = x.checked_add_signed(dx).unwrap_or(limit - 1);
				if x == limit {
					x = 0
				}
				x
			};
			(move_wrapping(i, *dx, h), move_wrapping(j, *dy, w))
		});

		let neighbor_count: u8 = neighbors.map(|pos| {
			match buff[pos] {
				Cell::Alive => 1,
				Cell::Dead => 0,
			}
		}).sum();

		match buff[(i, j)] {
			// Birth
			Cell::Dead if neighbor_count == 3 => Cell::Alive,
			// Death by isolation
			Cell::Alive if neighbor_count < 2 => Cell::Dead,
			// Death by overpopulation
			Cell::Alive if neighbor_count > 3 => Cell::Dead,
			// Stable
			Cell::Alive => Cell::Alive,
			Cell::Dead => Cell::Dead,
		}
	}

	fn new(start: Buff) -> Self {
		let (w, h) = start.dims();
		let other = Buff::new(w, h, Cell::Dead);
		Self {
			buffs: (start, other),
			epoch_parity: true,
		}
	}

	fn update(&mut self) {
		let (prev, next) = {
			if self.epoch_parity {
				(&self.buffs.0, &mut self.buffs.1)
			} else {
				(&self.buffs.1, &mut self.buffs.0)
			}
		};
		self.epoch_parity = !self.epoch_parity;

		let (w, h) = prev.dims();
		for i in 0..h {
			for j in 0..w {
				next[(i, j)] = Self::cell_evolution(prev, i, j);
			}
		}
	}

	fn state(&self) -> &Buff {
		if self.epoch_parity {
			&self.buffs.0
		} else {
			&self.buffs.1
		}
	}
}

trait Renderer {
	fn size(&self) -> (usize, usize);
	fn render(&mut self, b: &Buff);
}

struct TerminalRenderer {
	stdout: std::io::Stdout,
}

impl Default for TerminalRenderer {
	fn default() -> Self {
		Self {
			stdout: std::io::stdout()
		}
	}
}

impl Renderer for TerminalRenderer {
	fn size(&self) -> (usize, usize) {
		let (w, h) = crossterm::terminal::size().unwrap();
		(w.try_into().unwrap(), h.try_into().unwrap())
	}

	fn render(&mut self, b: &Buff) {
		use crossterm::*;
		let (w, h) = b.dims();

		_ = self.stdout.queue(cursor::Hide);
		for i in 0..h {
			_= queue!(self.stdout, cursor::MoveTo(0, i.try_into().unwrap()));
			for j in 0..w {
				_ = queue!(self.stdout,
					style::Print(if let Cell::Alive = b[(i, j)] { '█' } else { ' ' }),
				);

			}
		}

		_ = self.stdout.queue(cursor::Show);
		self.stdout.flush().unwrap();
	}
}

fn main() {
	const ALIVE_PROB: f64 = 0.2;

	let mut renderer = TerminalRenderer::default();
	let (w, h) = renderer.size();
	let mut gol = GameOfLife::new(random_buff(w, h, ALIVE_PROB));

	loop {
		gol.update();

		let b = gol.state();
		renderer.render(b);

		sleep(Duration::from_millis(50));
	}
}
