use axum::{
    response::Html,
    routing::{get, post},
    Form, Router,
};

use serde::Deserialize;

use tokio::{fs::File, io::AsyncWriteExt, process::Command};
use tower_http::services::ServeDir;

static TEX_BEGIN: &str = r"
\documentclass[border=1pt,preview]{standalone}
\usepackage{amsmath}
\begin{document}";
static TEX_END: &str = r"\end{document}";

async fn root() -> Html<&'static str> {
    Html(include_str!("../index.html"))
}

async fn convert(Form(form): Form<Latex>) -> Html<String> {
    let id = uuid::Uuid::new_v4().to_string();

    let tex_file_name = format!("{}.tex", id);
    let tex_path = format!("tmp/{}", tex_file_name);

    let mut file = File::create(tex_path.as_str()).await.unwrap();

    let tex = format!("{TEX_BEGIN}\n${}$\n{TEX_END}", form.latex);
    file.write(tex.as_bytes()).await.unwrap();

    let mut command = Command::new("pdflatex");
    command.args(&["-halt-on-error", "-output-directory=tmp", tex_path.as_str()]);
    if !command.output().await.unwrap().status.success() {
        let log = tokio::fs::read_to_string(format!("tmp/{id}.log"))
            .await
            .unwrap();
        let html = format!(
            "
    <div class=\"card p-5\">
    <div class=\"alert alert-danger\" role=\"alert\">
    Error: could not compile file
    </div>

    <textarea class=\"form-control border-danger\" rows=\"20\" readonly>{log}</textarea>
    </div>
        "
        );
        return Html(html);
    }

    let out_pdf = format!("out/{id}.pdf");
    let out_png = format!("out/{id}.png");

    tokio::fs::rename(format!("tmp/{id}.pdf"), out_pdf.as_str())
        .await
        .unwrap();

    let mut convert_command = Command::new("convert");
    convert_command.args([
        "-density",
        "900",
        "-units",
        "PixelsPerInch",
        "-quality",
        "90",
        out_pdf.as_str(),
        out_png.as_str(),
    ]);

    if !convert_command.output().await.unwrap().status.success() {
        todo!("Handle error!");
    }

    let html = format!("
    <div class=\"card p-5\">
    <div class=\"mb-4 px-0\">
    <figure>
    <img src=\"{out_png}\" class=\"img-fluid\">
    </figure>
    <div class=\"row\">
    <div class=\"col xs-1\" align=\"center\">
    <a href=\"{out_png}\" download=\"out.png\" type=\"button\" class=\"btn btn-dark\">Download png</a>
    </div>
    <div class=\"col xs-1\" align=\"center\">
    <a href=\"{out_pdf}\" download=\"out.pdf\" type=\"button\" class=\"btn btn-dark\">Download pdf</a>
    </div>
    </div>
    ");

    Html(html)
}

#[derive(Deserialize, Debug)]
struct Latex {
    latex: String,
}

#[tokio::main]
async fn main() {
    tokio::fs::create_dir_all("tmp").await.unwrap();
    tokio::fs::create_dir_all("out").await.unwrap();
    let app = Router::new()
        .route("/", get(root))
        .route("/convert", post(convert))
        .nest_service("/out", ServeDir::new("out/"));
    let addr = "0.0.0.0:3000".parse().unwrap();

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
