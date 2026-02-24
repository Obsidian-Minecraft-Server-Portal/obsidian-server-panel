//! SQL placeholder conversion for cross-database compatibility.
//!
//! Write queries using `?` placeholders. For PostgreSQL, they are
//! automatically converted to `$1`, `$2`, etc. For SQLite and MySQL
//! the query is returned unchanged.

use std::borrow::Cow;

/// Converts `?` parameter placeholders to the format required by the
/// active database backend.
///
/// - **SQLite / MySQL:** Returns the query unchanged (borrows).
/// - **PostgreSQL:** Replaces each `?` with `$1`, `$2`, ... (allocates).
///
/// # Example
/// ```
/// use obsidian_database::sql;
///
/// let q = sql("SELECT * FROM users WHERE id = ? AND name = ?");
/// // SQLite/MySQL: "SELECT * FROM users WHERE id = ? AND name = ?"
/// // PostgreSQL:   "SELECT * FROM users WHERE id = $1 AND name = $2"
/// ```
#[cfg(any(feature = "sqlite", feature = "mysql"))]
#[inline]
pub fn sql(query: &str) -> Cow<'_, str> {
    Cow::Borrowed(query)
}

/// Converts `?` parameter placeholders to `$1, $2, ...` for PostgreSQL.
#[cfg(feature = "postgres")]
pub fn sql(query: &str) -> Cow<'_, str> {
    if !query.contains('?') {
        return Cow::Borrowed(query);
    }
    let mut result = String::with_capacity(query.len() + 32);
    let mut n = 1u32;
    for ch in query.chars() {
        if ch == '?' {
            use std::fmt::Write;
            let _ = write!(result, "${n}");
            n += 1;
        } else {
            result.push(ch);
        }
    }
    Cow::Owned(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_placeholders_borrows() {
        let result = sql("SELECT * FROM users");
        assert_eq!(&*result, "SELECT * FROM users");
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn with_placeholders() {
        let result = sql("SELECT * FROM users WHERE id = ? AND name = ?");
        #[cfg(any(feature = "sqlite", feature = "mysql"))]
        assert_eq!(&*result, "SELECT * FROM users WHERE id = ? AND name = ?");
        #[cfg(feature = "postgres")]
        assert_eq!(
            &*result,
            "SELECT * FROM users WHERE id = $1 AND name = $2"
        );
    }

    #[test]
    fn empty_query() {
        let result = sql("");
        assert_eq!(&*result, "");
        assert!(matches!(result, Cow::Borrowed(_)));
    }
}
