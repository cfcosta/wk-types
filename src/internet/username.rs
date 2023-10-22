use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
    str::FromStr,
};

use crate::{testing::from_regex, Error, Kind};

#[cfg(feature = "testing")]
use crate::testing::NegateArbitrary;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[repr(transparent)]
/// A handle for an user, with only alphanumeric characters and underscores
pub struct Username(String);

impl FromStr for Username {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use chumsky::{error::Cheap, prelude::*};

        let result = filter::<_, _, Cheap<char>>(|&c: &char| c.is_ascii_alphanumeric())
            .map(Some)
            .chain::<char, Vec<_>, _>(
                filter(|c: &char| c.is_ascii_alphanumeric() || *c == '_').repeated(),
            )
            .then_ignore(end())
            .collect()
            .parse(s)
            .map_err(|_| Error::FailedParsing(Kind::Username, s.to_string()))?;

        Ok(Self(result))
    }
}

impl Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl Deref for Username {
    type Target = <String as Deref>::Target;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Username {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "testing")]
impl<'a> arbitrary::Arbitrary<'a> for Username {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        use fake::{faker::internet::en::Username, Fake};
        use rand::{rngs::StdRng, SeedableRng};

        let mut rng = StdRng::from_seed(u.arbitrary()?);
        let user = Username()
            .fake_with_rng::<String, _>(&mut rng)
            .replace('.', "_");

        Ok(Self(user))
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        String::size_hint(depth)
    }
}

#[cfg(feature = "testing")]
impl<'a> NegateArbitrary<'a> for Username {
    fn negate_arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let gen = from_regex(u, "[^a-zA-Z0-9].*|.*[^a-zA-Z0-9_].*");
        Ok(Self(gen.get()))
    }
}

#[cfg(all(test, feature = "testing"))]
mod tests {
    use proptest::*;

    use crate::testing::*;

    use super::*;

    proptest! {
        #[test]
        fn parses_correctly(u in "[a-zA-Z0-9][a-zA-Z0-9_]*") {
            u.to_string().parse::<Username>().expect("Failed parsing");
        }

        #[test]
        fn rejects_all_invalid(u in "[^a-zA-Z0-9].*|.*[^a-zA-Z0-9_].*") {
            assert!(u.parse::<Username>().is_err())
        }

        #[test]
        fn arbitrary_email_is_always_valid(a in gen::<Username>()) {
            a.to_string().parse::<Username>().expect("Failed parsing");
        }

        #[test]
        fn invalid_emails_are_always_invalid(a in invalid::<Username>()) {
            assert_eq!(
                a.to_string().parse::<Username>(),
                Err(Error::FailedParsing(Kind::Username, a.to_string()))
            );
        }
    }
}
