pub trait VecUtils<T> {
    fn unshift(&mut self,v: T);
    fn shift(&mut self) -> T;
    /// Replaces old Iterator::count
    fn count(&self,f: &Fn(&T) -> bool) -> usize;
}


impl<T> VecUtils<T> for Vec<T> {
    fn unshift(&mut self,v: T) {
        self.insert(0,v);
    }
    fn shift(&mut self) -> T {
        self.remove(0)
    }
    /// Replaces old Iterator::count
    fn count(&self,f: &Fn(&T) -> bool) -> usize {
        let mut count = 0;
        for value in self.iter() {
            if f(value) {
                count += 1;
            }
        }
        count
    }
}

