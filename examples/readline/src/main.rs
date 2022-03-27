#![feature(try_blocks)]

use async_std::{io::{self, stdin}, stream, task};
use rustyline_async::{Readline, ReadlineError};

use std::{time::Duration, io::Write};

use futures::prelude::*;

#[async_std::main]
async fn main() -> Result<(), ReadlineError> {
	let mut periodic_timer1 = stream::interval(Duration::from_secs(2));
	let mut periodic_timer2 = stream::interval(Duration::from_secs(3));

	let (mut rl, writer) = Readline::new("> ".to_owned(), stdin()).unwrap();

	#[derive(Clone)]
	struct AsyncWriteWrapper<W: AsyncWrite + Unpin + Send + Clone>(W);
	impl<W: AsyncWrite + Unpin + Send + Clone> std::io::Write for AsyncWriteWrapper<W> {
		fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
			let ret = task::block_on(self.0.write(buf))?;
			task::block_on(self.0.flush())?;
			Ok(ret)
		}

		fn flush(&mut self) -> io::Result<()> {
			task::block_on(self.0.flush())
		}
	}

	let mut stdout = AsyncWriteWrapper(writer.clone());
	simplelog::WriteLogger::init(log::LevelFilter::Debug, simplelog::Config::default(), stdout.clone()).unwrap();


	let join = task::spawn(async move {
		
		let ret: Result<(), ReadlineError> = try { loop {
			futures::select! {
				_ = periodic_timer1.next().fuse() => {
					write!(stdout, "First timer went off!")?;
				}
				_ = periodic_timer2.next().fuse() => {
					//write!(stdout_2, "Second timer went off!")?;
					task::spawn_blocking(||log::info!("Second timer went off!"));
					
				}
				command = rl.readline().fuse() => if let Some(command) = command {
					match command {
						Ok(line) => write!(stdout, "Received line: {}", line)?,
						Err(ReadlineError::Eof) =>{ write!(stdout, "Exiting...")?; break },
						Err(ReadlineError::Interrupted) => write!(stdout, "CTRL-C")?,
						Err(err) => {
							write!(stdout, "Received err: {:?}", err)?;
							break;
						},
					}
				}
			}
			rl.flush()?;
		}};
		ret
	});
	
	println!("Exited with: {:?}", join.await);
	Ok(())
}
