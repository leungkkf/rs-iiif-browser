use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
/// One object of type "S" or many objects of type "T".
pub(crate) enum OneOrMany<S, T> {
    One(S),
    Many(Vec<T>),
}

/// One object of type "T" or many objects of type "T".
pub(crate) type OneTypeOrMany<T> = OneOrMany<T, T>;

impl<T> OneTypeOrMany<T> {
    /// Return the non-consuming iterator.
    pub(crate) fn iter(&self) -> <&'_ OneTypeOrMany<T> as IntoIterator>::IntoIter {
        self.into_iter()
    }
}

/// IntoIterator trait for the non-consuming iterator to iterate through the items in OneTypeOrMany.
impl<'a, T> IntoIterator for &'a OneTypeOrMany<T> {
    type Item = &'a T;
    type IntoIter = Box<dyn Iterator<Item = &'a T> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            OneOrMany::One(v) => Box::new(Some(v).into_iter()),
            OneOrMany::Many(v) => Box::new(v.iter()),
        }
    }
}

/// IntoIterator trait for the consuming iterator to iterate through the items in OneTypeOrMany.
impl<T> IntoIterator for OneTypeOrMany<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            OneOrMany::One(v) => vec![v].into_iter(),
            OneOrMany::Many(vec) => vec.into_iter(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_type_or_many_into_iter() {
        let test_string = "test";

        let test_data = OneTypeOrMany::One(test_string.to_string());

        let actual: Vec<_> = (&test_data).into_iter().collect();
        assert_eq!(actual, vec![test_string]);

        let actual: Vec<_> = test_data.into_iter().collect();
        assert_eq!(actual, vec![test_string]);

        let test_data = OneTypeOrMany::Many(vec![test_string.to_string(), test_string.to_string()]);

        let actual: Vec<_> = (&test_data).into_iter().collect();
        assert_eq!(actual, vec![test_string, test_string]);

        let actual: Vec<_> = test_data.into_iter().collect();
        assert_eq!(actual, vec![test_string, test_string]);
    }
}
