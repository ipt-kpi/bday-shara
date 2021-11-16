use warp::Reply;

use crate::model::user::AuthInfo;
use crate::Application;

pub async fn prizes(
    _app: &'static Application,
    _auth_info: AuthInfo,
) -> Result<impl Reply, warp::Rejection> {
    Ok(warp::reply::json(&serde_json::json!({})))
}