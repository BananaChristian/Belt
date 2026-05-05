#[derive(Debug)]
pub enum BuildMode{
    Executable,
    Static,
    Object,
}

#[derive(Debug)]
pub struct Project{
    pub name: String,
    pub version: String,
    pub entry: Option<String>,
    pub mode: BuildMode
}

#[derive(Debug)]
pub struct Layout{
    pub src: String,
    pub stubs: String,
    pub objs: String,
    pub build: String
}

#[derive(Debug)]
pub struct Dependecies{
    pub deps: Vec<(String,String)>,
}

#[derive(Debug)]
pub struct Link{
    pub links:Vec<(String,Vec<String>)>
}

#[derive(Debug)]
pub struct Config{
    pub project: Project,
    pub layout: Layout,
    pub dependecies: Option<Dependecies>,
    pub link: Option<Link>
}

