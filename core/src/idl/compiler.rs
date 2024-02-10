pub struct Compiler {}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {}
    }

    pub fn compile(&self, input: &str) -> Result<String, String> {
        Ok(input.to_string())
    }
}