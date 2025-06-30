/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   api.rs                                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: maiboyer <maiboyer@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2025/06/30 14:49:21 by maiboyer          #+#    #+#             */
/*   Updated: 2025/06/30 16:16:43 by maiboyer         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

pub mod client;

#[macro_export]
macro_rules! _try_id_default {
    (@default onnone) => {{
        return StatusCode::NOT_FOUND;
    }};
    (@default onerr) => {{
        return StatusCode::INTERNAL_SERVER_ERROR;
    }};
}

#[macro_export]
macro_rules! _try_id_inner {
    // start of 2
    (@inner {
        database: $database:expr,
        ty: $ty:ty,
        raw: $raw:expr,
        onnone: $none:expr,
        onerr: $err:expr,
    }) => {{
        match <$ty>::from_raw($database, $raw).await {
            Ok(Some(v)) => v,
            Ok(None) => $none,
            Err(e) => {::log::error!("database error: {e}"); $err},
        }
    }};

    (@inner {
        database: $database:expr,
        ty: $ty:ty,
        raw: $raw:expr,
        onnone: ,
        onerr: ,
    }) => {{
        $crate::_try_id_inner!(@inner {
            database: $database,
            ty: $ty,
            raw: $raw,
            onnone: $crate::_try_id_default!(@default onnone),
            onerr: $crate::_try_id_default!(@default onerr),
        })
    }};
    // end of 2

    // start of 1
    (@inner {
        database: $database:expr,
        ty: $ty:ty,
        raw: $raw:expr,
        onnone: ,
        onerr: $err:expr,
    }) => {{
        $crate::_try_id_inner!(@inner {
            database: $database,
            ty: $ty,
            raw: $raw,
            onnone: $crate::_try_id_default!(@default onnone),
            onerr: $err,
        })
    }};

    (@inner {
        database: $database:expr,
        ty: $ty:ty,
        raw: $raw:expr,
        onnone: $none:expr,
        onerr: ,
    }) => {{
        $crate::_try_id_inner!(@inner {
            database: $database,
            ty: $ty,
            raw: $raw,
            onnone: $none,
            onerr: $crate::_try_id_default!(@default onerr),
        })
    }};
}

#[macro_export]
macro_rules! try_id {
    (<$ty:ty>($database:expr, $raw:expr)$(; $(@err $err:block)? $(@none $none:block)?)?) => {
        $crate::_try_id_inner!(@inner {
            database: $database,
            ty: $ty,
            raw: $raw,
            onnone: $($({$none})?)? ,
            onerr: $($({$err})?)? ,
        })
    };

}

pub use _try_id_default;
pub use _try_id_inner;
pub use try_id;
