use std::{io::Write, thread::sleep, time::Duration};

#[derive(Clone, Copy, Debug)]
enum Cell {
	Dead,
	Alive,
}

struct Config {
	w: usize,
	h: usize,
	alive_probability: f64,
}

impl Default for Config {
	fn default() -> Self {
		Config {
			w: 100,
			h: 30,
			alive_probability: 0.2,
		}
	}
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

fn empty_buff(cfg: &Config) -> Buff {
	Buff::new(cfg.w, cfg.h, Cell::Dead)
}

fn random_buff(cfg: &Config) -> Buff {
	use rand::prelude::*;

	let mut rng = rand::thread_rng();
	let mut random_state = || {
		match rng.gen_bool(cfg.alive_probability) {
			true => Cell::Alive,
			false => Cell::Dead,
		}
	};

	Buff::new_with(cfg.w, cfg.h, |_i, _j| random_state())
}

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

fn main() {
	let cfg = Config::default();
	let mut buffs = (random_buff(&cfg), empty_buff(&cfg));

	let mut epoch = 0;
	let mut stdout = std::io::stdout();

	loop {

		// padding
		for _ in 0..5 {
			_ = write!(stdout, "\n");
		}

		let (prev, next) = {
			if epoch % 2 == 0 {
				(&buffs.0, &mut buffs.1)
			} else {
				(&buffs.1, &mut buffs.0)
			}
		};
		epoch += 1;

		let (w, h) = (cfg.w, cfg.h);
		for i in 0..h {
			for j in 0..w {
				next[(i, j)] = cell_evolution(prev, i, j);
				_ = write!(stdout, "{}", if let Cell::Alive = prev[(i, j)] { 'o' } else { ' ' });
			}
			_ = write!(stdout, "\n");
		}

		stdout.flush().unwrap();
		sleep(Duration::from_millis(500));

	}

}
