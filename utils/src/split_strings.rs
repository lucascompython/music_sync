pub struct SplitStrings<'a> {
    input: &'a str,
    delimiter: char,
}

impl<'a> SplitStrings<'a> {
    pub fn new(input: &'a str, delimiter: char) -> Self {
        SplitStrings { input, delimiter }
    }
}

impl<'a> Iterator for SplitStrings<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.input.is_empty() {
            None
        } else {
            if let Some(pos) = self.input.find(self.delimiter) {
                let (head, tail) = self.input.split_at(pos);
                self.input = &tail[1..]; // Skip past the delimiter
                Some(head.to_string())
            } else {
                let result = self.input.to_string();
                self.input = "";
                Some(result)
            }
        }
    }
}
