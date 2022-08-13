use crate::{
    error::{Error, Result},
    AppState,
};
use axum::extract::Extension;
use axum_extra::extract::cookie::{Cookie, CookieJar};
use time::{Duration as CookieDuration, OffsetDateTime};
use tokio::time::{sleep, Duration};
use uuid::Uuid;

pub async fn setup(
    Extension(state): Extension<AppState>,
    jar: CookieJar,
) -> Result<CookieJar, Error> {
    let pool = &state.pool;

    let valid_user_id = {
        if let Some(user_id_str) = jar.get("user_id").map(|cookie| cookie.value().to_owned()) {
            if let Ok(user_id) = Uuid::parse_str(&user_id_str) {
                sqlx::query!("select id from identity.users where id = $1", user_id)
                    .fetch_optional(pool)
                    .await?
            } else {
                None
            }
        } else {
            None
        }
    };

    if valid_user_id.is_none() {
        let user_id = Uuid::new_v4();
        sqlx::query!("insert into identity.users (id) values ($1)", user_id)
            .execute(pool)
            .await?;

        let mut cookie = Cookie::build("user_id", user_id.to_string())
            .path("/")
            // .secure(true)
            // .http_only(true)
            .finish();

        let expiration_time = OffsetDateTime::now_utc() + CookieDuration::hours(24 * 365 * 5);

        cookie.set_expires(expiration_time);

        sleep(Duration::from_millis(1000)).await;

        Ok(jar.add(cookie))
    } else {
        Ok(jar)
    }
}
