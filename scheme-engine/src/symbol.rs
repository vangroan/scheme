use crate::declare_id;

declare_id!(pub struct SymbolId(u16));

#[derive(Debug)]
pub struct SymbolTable {
    symbols: Vec<String>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
        }
    }

    pub fn resolve(&self, name_query: impl AsRef<str>) -> Option<SymbolId> {
        let name_query = name_query.as_ref();
        self.symbols
            .iter()
            .position(|name| name.as_str() == name_query)
            .map(|index| SymbolId(index as u16))
    }

    pub fn intern_symbol(&mut self, name: impl ToString) -> SymbolId {
        let name = name.to_string();

        match self.resolve(name.as_str()) {
            Some(symbol) => symbol,
            None => {
                let next_index = self.symbols.len();
                self.symbols.push(name);
                SymbolId(next_index as u16)
            }
        }
    }

    pub fn insert_unique(&mut self, name: impl ToString) -> Option<SymbolId> {
        let name = name.to_string();

        match self.resolve(name.as_str()) {
            Some(_) => None,
            None => {
                let next_index = self.symbols.len();
                self.symbols.push(name);
                Some(SymbolId(next_index as u16))
            }
        }
    }

    pub fn items(&self) -> impl Iterator<Item = (SymbolId, &str)> {
        self.symbols
            .iter()
            .enumerate()
            .map(|(index, name)| (SymbolId(index as u16), name.as_str()))
    }
}
