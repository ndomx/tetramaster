use std::io;

pub trait AsciiView {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn line(&self, line: usize) -> String;

    fn render(&self) -> io::Result<()> {
        for line in 0..self.height() {
            println!("{}", self.line(line));
        }

        Ok(())
    }
}
