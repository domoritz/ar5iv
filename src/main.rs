#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use rocket::fs::NamedFile;
use rocket::http::ContentType;
use rocket::http::Status;
use rocket::response::{content, status, Redirect};
use rocket::Request;
use rocket_db_pools::Connection;
use rocket_db_pools::{deadpool_redis, Database};
use rocket_dyn_templates::Template;

use ar5iv::cache::{assemble_paper_asset_with_cache, assemble_paper_with_cache};
use std::collections::HashMap;
use std::path::Path;

#[macro_use]
extern crate lazy_static;
use regex::Regex;
lazy_static! {
  static ref TRAILING_PDF_EXT: Regex = Regex::new("[.]pdf$").unwrap();
}

#[derive(Database)]
#[database("memdb")]
pub struct Cache(deadpool_redis::Pool);

#[get("/")]
async fn about() -> Template {
  let map: HashMap<String, String> = HashMap::new();
  Template::render("ar5iv", &map)
}

#[get("/favicon.ico")]
async fn favicon() -> Option<NamedFile> {
  NamedFile::open(Path::new("assets/").join("favicon.ico"))
    .await
    .ok()
}

#[get("/html/<id>")]
async fn get_html(mut conn: Connection<Cache>, id: &str) -> content::RawHtml<String> {
  content::RawHtml(assemble_paper_with_cache(&mut conn, None, id).await)
}
#[get("/html/<field>/<id>")]
async fn get_field_html(
  mut conn: Connection<Cache>,
  field: String,
  id: &str,
) -> content::RawHtml<String> {
  let paper = assemble_paper_with_cache(&mut conn, Some(field), id).await;
  content::RawHtml(paper)
}

#[get("/html/<id>/assets/<filename>")]
async fn get_paper_asset(
  mut conn: Connection<Cache>,
  id: &str,
  filename: &str,
) -> Option<(ContentType, Vec<u8>)> {
  assemble_paper_asset_with_cache(&mut conn, None, id, filename).await
}
#[get("/html/<field>/<id>/assets/<filename>", rank = 2)]
async fn get_field_paper_asset(
  mut conn: Connection<Cache>,
  field: String,
  id: &str,
  filename: &str,
) -> Option<(ContentType, Vec<u8>)> {
  assemble_paper_asset_with_cache(&mut conn, Some(field), id, filename).await
}
#[get("/html/<id>/assets/<subdir>/<filename>")]
async fn get_paper_subdir_asset(
  mut conn: Connection<Cache>,
  id: &str,
  subdir: String,
  filename: &str,
) -> Option<(ContentType, Vec<u8>)> {
  let compound_name = subdir + "/" + filename;
  assemble_paper_asset_with_cache(&mut conn, None, id, &compound_name).await
}
#[get("/html/<id>/assets/<subdir>/<subsubdir>/<filename>")]
async fn get_paper_subsubdir_asset(
  mut conn: Connection<Cache>,
  id: &str,
  subdir: String,
  subsubdir: &str,
  filename: &str,
) -> Option<(ContentType, Vec<u8>)> {
  let compound_name = subdir + "/" + subsubdir + "/" + filename;
  assemble_paper_asset_with_cache(&mut conn, None, id, &compound_name).await
}
#[get("/html/<field>/<id>/assets/<subdir>/<filename>", rank = 2)]
async fn get_field_paper_subdir_asset(
  mut conn: Connection<Cache>,
  field: String,
  id: &str,
  subdir: String,
  filename: &str,
) -> Option<(ContentType, Vec<u8>)> {
  let compound_name = subdir + "/" + filename;
  assemble_paper_asset_with_cache(&mut conn, Some(field), id, &compound_name).await
}
#[get("/html/<field>/<id>/assets/<subdir>/<subsubdir>/<filename>", rank = 2)]
async fn get_field_paper_subsubdir_asset(
  mut conn: Connection<Cache>,
  field: String,
  id: &str,
  subdir: String,
  subsubdir: &str,
  filename: &str,
) -> Option<(ContentType, Vec<u8>)> {
  let compound_name = subdir + "/" + subsubdir + "/" + filename;
  assemble_paper_asset_with_cache(&mut conn, Some(field), id, &compound_name).await
}

#[get("/abs/<field>/<id>")]
async fn abs_field(field: &str, id: &str) -> Redirect {
  let to_uri = String::from("/html/") + field + "/" + id;
  Redirect::to(to_uri)
}
#[get("/abs/<id>")]
async fn abs(id: &str) -> Redirect {
  let to_uri = String::from("/html/") + id;
  Redirect::to(to_uri)
}

#[get("/pdf/<field>/<id>")]
async fn pdf_field(field: &str, id: String) -> Redirect {
  let id_core: String = (*TRAILING_PDF_EXT.replace(&id, "")).to_owned();
  let to_uri = String::from("/html/") + field + "/" + &id_core;
  Redirect::to(to_uri)
}
#[get("/pdf/<id>")]
async fn pdf(id: String) -> Redirect {
  let id_core: String = (*TRAILING_PDF_EXT.replace(&id, "")).to_owned();
  let to_uri = String::from("/html/") + &id_core;
  Redirect::to(to_uri)
}

#[get("/assets/<name>")]
async fn assets(name: String) -> Option<NamedFile> {
  NamedFile::open(Path::new("assets/").join(name)).await.ok()
}

#[catch(404)]
fn general_not_found() -> content::RawHtml<&'static str> {
  content::RawHtml(
    r#"
        <p>Hmm... What are you looking for?</p>
        Say <a href="/hello/Sergio/100">hello!</a>
    "#,
  )
}

#[catch(default)]
fn default_catcher(status: Status, req: &Request<'_>) -> status::Custom<String> {
  let msg = format!("{} ({})", status, req.uri());
  status::Custom(status, msg)
}

#[launch]
fn rocket() -> _ {
  rocket::build()
    .attach(Template::fairing())
    .attach(Cache::init())
    .mount(
      "/",
      routes![
        abs,
        abs_field,
        pdf,
        pdf_field,
        get_html,
        get_field_html,
        get_paper_asset,
        get_paper_subdir_asset,
        get_paper_subsubdir_asset,
        get_field_paper_asset,
        get_field_paper_subdir_asset,
        get_field_paper_subsubdir_asset,
        about,
        assets,
        favicon
      ],
    )
    .register("/", catchers![general_not_found, default_catcher])
}