use crate::user::models::{AuthToken, TeamMember, TeamUser, User, UserProperties, OTP, BackupUser};
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel::MysqlConnection;
use crate::get_current_time;

pub fn add_new_user(user: &User, conn: &MysqlConnection) -> Result<(), diesel::result::Error> {
    use crate::schema::users::dsl::*;
    diesel::insert_into(users)
        .values(user)
        .execute(conn)
        .unwrap();
    Ok(())
}

pub fn get_user_by_name(
    user: &str,
    conn: &MysqlConnection,
) -> Result<Option<User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;
    users
        .filter(username.eq(user))
        .first::<User>(conn)
        .optional()
}

pub fn get_id_by_name(
    user: &str,
    conn: &MysqlConnection,
) -> Result<Option<i64>, diesel::result::Error> {
    use crate::schema::users::dsl::*;
    users
        .select(id)
        .filter(username.eq(user))
        .first(conn)
        .optional()
}

pub fn get_user_by_id(
    l_id: &i64,
    conn: &MysqlConnection,
) -> Result<Option<User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;
    users.filter(id.eq(l_id)).first::<User>(conn).optional()
}

pub fn get_found_users(conn: &MysqlConnection) -> Result<Vec<User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;
    users.filter(status.eq("Found")).load::<User>(conn)
}

pub fn get_users(conn: &MysqlConnection) -> Result<Vec<User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;

    users.load::<User>(conn)
}

pub fn delete_user(us: &i64, conn: &MysqlConnection) -> Result<(), diesel::result::Error> {
    use crate::schema::users::dsl::*;

    diesel::delete(users.filter(id.eq(us)))
        .execute(conn)
        .unwrap();
    Ok(())
}

pub fn update_user(user: &User, conn: &MysqlConnection) -> Result<(), diesel::result::Error> {
    use crate::schema::users::dsl::*;

    diesel::update(users.filter(id.eq(user.id)))
        .set((
            password.eq(&user.password),
            status.eq(&user.status),
            status_changed.eq(&user.status_changed),
            reviewer.eq(&user.reviewer),
            properties.eq(&user.properties),
            discoverer.eq(&user.discoverer),
        ))
        .execute(conn)
        .unwrap();
    Ok(())
}

pub fn update_properties(
    user: &i64,
    props: UserProperties,
    conn: &MysqlConnection,
) -> Result<(), diesel::result::Error> {
    use crate::schema::users::dsl::*;

    diesel::update(users.filter(id.eq(user)))
        .set((properties.eq(&props), ))
        .execute(conn)?;
    Ok(())
}

pub fn update_title(
    user: &i64,
    tit: &str,
    conn: &MysqlConnection,
) -> Result<(), diesel::result::Error> {
    use crate::schema::users::dsl::*;

    diesel::update(users.filter(id.eq(user)))
        .set((title.eq(tit), ))
        .execute(conn)?;
    Ok(())
}

pub fn update_password(
    user: &i64,
    pass: String,
    conn: &MysqlConnection,
) -> Result<(), diesel::result::Error> {
    use crate::schema::users::dsl::*;

    diesel::update(users.filter(id.eq(user)))
        .set((password.eq(&pass), password_changed.eq(get_current_time())))
        .execute(conn)?;
    Ok(())
}

pub fn get_user_from_auth_token(
    token: String,
    conn: &MysqlConnection,
) -> Result<Option<User>, diesel::result::Error> {
    let result = get_auth_token(token, conn);
    if result.is_err() {
        return Err(result.err().unwrap());
    }
    let result = result.unwrap();
    if result.is_none() {
        return Ok(None);
    }
    let result = result.unwrap();
    get_user_by_id(&result.user, conn)
}

//Auth Token
pub fn get_auth_token(
    a_token: String,
    conn: &MysqlConnection,
) -> Result<Option<AuthToken>, diesel::result::Error> {
    use crate::schema::auth_tokens::dsl::*;
    let found_token = auth_tokens
        .filter(token.eq(a_token))
        .first::<AuthToken>(conn)
        .optional()?;
    Ok(found_token)
}

pub fn add_new_auth_token(
    t: &AuthToken,
    conn: &MysqlConnection,
) -> Result<(), diesel::result::Error> {
    use crate::schema::auth_tokens::dsl::*;
    diesel::insert_into(auth_tokens)
        .values(t)
        .execute(conn)
        .unwrap();
    Ok(())
}

pub fn get_opt(value: &str, conn: &MysqlConnection) -> Result<Option<OTP>, diesel::result::Error> {
    use crate::schema::otps::dsl::*;
    let x: Option<OTP> = otps
        .filter(password.eq(value))
        .first::<OTP>(conn)
        .optional()?;
    Ok(x)
}

pub fn delete_otp(otp_id: i64, conn: &MysqlConnection) -> Result<(), DieselError> {
    use crate::schema::otps::dsl::*;
    diesel::delete(otps).filter(id.eq(otp_id)).execute(conn)?;
    Ok(())
}

pub fn opt_exist(value: &str, conn: &MysqlConnection) -> Result<bool, diesel::result::Error> {
    use crate::schema::otps::dsl::*;
    let x: Option<i64> = otps
        .select(id)
        .filter(password.eq(value))
        .first(conn)
        .optional()?;
    Ok(x.is_some())
}

pub fn add_opt(value: &OTP, conn: &MysqlConnection) -> Result<(), DieselError> {
    use crate::schema::otps::dsl::*;

    diesel::insert_into(otps).values(value).execute(conn)?;
    Ok(())
}

pub fn get_team_members(conn: &MysqlConnection) -> Result<Vec<TeamMember>, DieselError> {
    use crate::schema::team_members::dsl::*;

    team_members.load::<TeamMember>(conn)
}

pub fn get_team_member(u: &i64, conn: &MysqlConnection) -> Result<Option<TeamMember>, DieselError> {
    use crate::schema::team_members::dsl::*;

    team_members
        .filter(user.eq(u))
        .first::<TeamMember>(conn)
        .optional()
}

pub fn get_team_user(
    user: &i64,
    conn: &MysqlConnection,
) -> Result<Option<TeamUser>, diesel::result::Error> {
    use crate::schema::users::dsl::*;
    users
        .filter(id.eq(user))
        .select((id, username, properties))
        .first::<TeamUser>(conn)
        .optional()
}

pub fn get_users_for_backup(conn: &MysqlConnection) -> Result<Vec<BackupUser>, diesel::result::Error> {
    use crate::schema::users::dsl::*;
    users.select((username, permissions, status, status_changed, discoverer, reviewer, properties, title, birthday, created)).load::<BackupUser>(conn)
}
