use color_eyre::Result;

use crate::database::{clientkeys::ClientKeyId, clients::ClientId, keys::KeyId, users::UserId};

pub async fn client_from_raw(
    db: &crate::database::Database,
    client: i64,
) -> Result<Option<crate::database::clients::TableClients>> {
    let Some(client) = ClientId::from_raw(db, client).await? else {
        return Ok(None);
    };
    db.fetch_client(client).await
}

pub async fn user_from_raw(
    db: &crate::database::Database,
    user: i64,
) -> Result<Option<crate::database::users::TableUsers>> {
    let Some(user) = UserId::from_raw(db, user).await? else {
        return Ok(None);
    };
    db.fetch_user(user).await
}

pub async fn key_from_raw(
    db: &crate::database::Database,
    key: i64,
) -> Result<Option<crate::database::keys::TableKeys>> {
    let Some(key) = KeyId::from_raw(db, key).await? else {
        return Ok(None);
    };
    db.fetch_key(key).await
}

pub async fn clientkey_from_raw(
    db: &crate::database::Database,
    clientkey: i64,
) -> Result<Option<crate::database::clientkeys::TableClientsKey>> {
    let Some(client_key) = ClientKeyId::from_raw(db, clientkey).await? else {
        return Ok(None);
    };
    db.fetch_client_key(client_key).await
}

pub async fn clientkey_from_client_and_key(
    db: &crate::database::Database,
    client: i64,
    key: i64,
) -> Result<Option<crate::database::clientkeys::TableClientsKey>> {
    let Some(client) = ClientId::from_raw(db, client).await? else {
        return Ok(None);
    };
    let Some(key) = KeyId::from_raw(db, key).await? else {
        return Ok(None);
    };
    db.fetch_client_key_from_client_and_key(client, key).await
}

// shamelessly stolen from `serde_with`
/// Makes a distinction between a missing, unset, or existing value
///
/// Some serialization formats make a distinction between missing fields, fields with a `null`
/// value, and existing values. One such format is JSON. By default it is not easily possible to
/// differentiate between a missing value and a field which is `null`, as they deserialize to the
/// same value. This helper changes it, by using an `Option<Option<T>>` to deserialize into.
///
/// * `None`: Represents a missing value.
/// * `Some(None)`: Represents a `null` value.
/// * `Some(Some(value))`: Represents an existing value.
///
/// Note: This cannot be made compatible to `serde_as`, since skipping of values is only available on the field level.
/// A hypothetical `DoubleOption<T>` with a `SerializeAs` implementation would allow writing something like this.
/// This cannot work, since there is no way to tell the `Vec` to skip the inner `DoubleOption` if it is `None`.
///
/// ```rust
/// # #[cfg(any())] {
/// # struct Foobar {
/// #[serde_as(as = "Vec<DoubleOption<_>>")]
/// data: Vec<Option<Option<i32>>>,
/// # }
/// # }
/// ```
///
/// # Examples
///
/// ```rust
/// # use serde::{Deserialize, Serialize};
/// #
/// # #[derive(Debug, PartialEq, Eq)]
/// #[derive(Deserialize, Serialize)]
/// struct Doc {
///     #[serde(
///         default,                                    // <- important for deserialization
///         skip_serializing_if = "Option::is_none",    // <- important for serialization
///         with = "::serde_with::rust::double_option",
///     )]
///     a: Option<Option<u8>>,
/// }
/// // Missing Value
/// let s = r#"{}"#;
/// assert_eq!(Doc { a: None }, serde_json::from_str(s).unwrap());
/// assert_eq!(s, serde_json::to_string(&Doc { a: None }).unwrap());
///
/// // Unset Value
/// let s = r#"{"a":null}"#;
/// assert_eq!(Doc { a: Some(None) }, serde_json::from_str(s).unwrap());
/// assert_eq!(s, serde_json::to_string(&Doc { a: Some(None) }).unwrap());
///
/// // Existing Value
/// let s = r#"{"a":5}"#;
/// assert_eq!(Doc { a: Some(Some(5)) }, serde_json::from_str(s).unwrap());
/// assert_eq!(s, serde_json::to_string(&Doc { a: Some(Some(5)) }).unwrap());
/// ```
#[allow(clippy::option_option)]
pub mod double_option {
    use serde::*;

    /// Deserialize potentially non-existing optional value
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
    where
        T: Deserialize<'de>,
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer).map(Some)
    }

    /// Serialize optional value
    pub fn serialize<S, T>(values: &Option<Option<T>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        match values {
            None => serializer.serialize_unit(),
            Some(None) => serializer.serialize_none(),
            Some(Some(v)) => serializer.serialize_some(&v),
        }
    }
}
