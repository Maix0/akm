/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   client.rs                                          :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: maiboyer <maiboyer@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2025/06/30 14:49:12 by maiboyer          #+#    #+#             */
/*   Updated: 2025/06/30 16:46:35 by maiboyer         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use axum::{
    extract::{Path, State},
    http::StatusCode,
};

use crate::{
    database::{clients::ClientId, keys::KeyId},
    try_id,
};

#[axum::debug_handler]
pub async fn client_id_key_id_newsecret(
    State(state): State<crate::AppState>,
    Path((client_id, key_id)): Path<(i64, i64)>,
) -> StatusCode {
    let client = try_id!(<ClientId>(&state.db, client_id));
    let key = try_id!(<KeyId>(&state.db, key_id));



    todo!()
}
