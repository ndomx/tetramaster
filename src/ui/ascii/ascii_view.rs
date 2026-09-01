pub trait AsciiView {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn line(&self, line: usize) -> String;
}
