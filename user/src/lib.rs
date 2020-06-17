#![allow(unused)]

#[macro_use]                                      
extern crate diesel;

pub mod schema;
pub mod models;
pub mod repos;



#[cfg(test)]
mod tests {
    #[test]
    fn add_login_info_test() {
        use super::models;
        use super::repos;
        use common::db_util;
        use diesel::prelude::*;
        use diesel::r2d2::{self, ConnectionManager};

        let new_login_info = models::NewLoginInfo {
            username: "zhangsan",
            password: "123456"
        };

        let conn: &MysqlConnection  = &db_util::POOL.get().unwrap();

        let count:  usize  =  repos::add_login_info( conn,   &new_login_info).expect("add login info error");
        println!("count:{}", count);
    }
}
