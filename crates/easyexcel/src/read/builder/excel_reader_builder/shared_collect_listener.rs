struct SharedCollectListener<T>(Rc<RefCell<Vec<T>>>);

impl<T> ReadListener<T> for SharedCollectListener<T> {
    fn invoke(&mut self, data: T, _context: &AnalysisContext) -> Result<()> {
        self.0.borrow_mut().push(data);
        Ok(())
    }
}

