use std::{
    fs::{self, File},
    io::{BufWriter, Write},
};

use crate::config::Layout;

fn create_lethr(project_name: &String) -> Result<(), std::io::Error> {
    let path = format!("{}/{}", project_name, "belt.lethr");
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "[project]")?;
    writeln!(writer, "name = \"{}\"", project_name)?;
    writeln!(writer, "version = \"0.1.0\"")?;
    writeln!(writer, "entry = \"src/main.unn\"")?;
    writeln!(writer, "mode = executable")?;

    writer.flush()?;

    Ok(())
}

fn create_example(path: &String) -> Result<(), std::io::Error> {
    let full_path = format!("{}/{}", path, "main.unn");
    let file = File::create(full_path)?;
    let mut writer = BufWriter::new(file);

    let content = r#"
    func main:void{
        trace "Hello world! "
    }
    "#;

    writer.write_all(content.as_bytes())?;
    writer.flush()?;
    Ok(())
}

fn create_project(project_name: &String, layout: &Layout) -> Result<(), std::io::Error> {
    //Create the parent directory
    fs::create_dir(project_name)?;

    //Create belt.lethr
    create_lethr(project_name)?;

    //Create the sub folders
    let src = format!("{}/{}", project_name, layout.src);
    let stubs = format!("{}/{}", project_name, layout.stubs);
    let objs = format!("{}/{}", project_name, layout.objs);
    let build = format!("{}/{}", project_name, layout.build);

    fs::create_dir_all(&src)?;
    fs::create_dir_all(stubs)?;
    fs::create_dir_all(objs)?;
    fs::create_dir_all(build)?;

    //Create the src/main.unn guy
    create_example(&src)?;

    Ok(())
}

pub fn scaffold(project_name: &String)->Result<(),std::io::Error>{
    let layout =Layout{
        src: "src".to_string(),
        stubs: "stubs".to_string(),
        objs: "objs".to_string(),
        build: "build".to_string(),
    };
    create_project(project_name, &layout)?;
    
    Ok(())
}
