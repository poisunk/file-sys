use file_sys::core::hardware;
use file_sys::core::fs;

fn main() {
    let hardware = hardware::Hardware::load("fs_data");
    let mut fs: fs::System = fs::System::init(hardware);

    main_cmd_loop(&mut fs);

    fs.save("fs_data");
}

fn main_cmd_loop(fs: &mut fs::System) {
    let mut current_dir = fs.get_root_dir();

    loop {
        let mut cmd = String::new();

        println!("{}>", current_dir.name);

        std::io::stdin()
            .read_line(&mut cmd)
            .expect("Failed to read line");

        let cmd = cmd.trim().split(" ").collect::<Vec<&str>>()  ;

        if cmd[0] == "exit" {
            break;
        }

        match cmd[0] {
            "ls" => {
                current_dir.show();
            }
            "cd" => {
                let mut dir_name = String::new();
                if cmd.len() > 1 {
                    dir_name = String::from(cmd[1]);
                }
                current_dir = fs.open_dir(&current_dir, dir_name.as_str());
            }
            "mkdir" => {
                let mut dir_name = String::new();
                if cmd.len() > 1 {
                    dir_name = String::from(cmd[1]);
                }
                fs.create_dir(&mut current_dir, dir_name.as_str());
            }
            "rmdir" => {
                let mut dir_name = String::new();
                if cmd.len() > 1 {
                    dir_name = String::from(cmd[1]);
                }
                fs.remove_dir(&mut current_dir, dir_name.as_str());
            }
            "create" => {
                let mut file_name = String::new();
                if cmd.len() > 1 {
                    file_name = String::from(cmd[1]);
                }
                fs.create_file(&mut current_dir, file_name.as_str());
            }
            "open" => {
                let mut file_name = String::new();
                if cmd.len() > 1 {
                    file_name = String::from(cmd[1]);
                }
                let file = fs.open_file(&mut current_dir, file_name.as_str());
                println!("{}", file.content);
            }
            "write" => {
                let mut file_name = String::new();
                if cmd.len() > 1 {
                    file_name = String::from(cmd[1]);
                }
                let mut file = fs.open_file(&mut current_dir, file_name.as_str());
                let mut content = String::new();
                if cmd.len() > 2 {
                    content = String::from(cmd[2]);
                }
                fs.write_file(&mut current_dir, &mut file, content.as_bytes());
                println!("{}", file.content);
            }
            "rm" => {
                let mut file_name = String::new();
                if cmd.len() > 1 {
                    file_name = String::from(cmd[1]);
                }
                fs.remove_file(&mut current_dir, file_name.as_str());
            }
            _ => {
                println!("未知命令");
            }
        }
    }
}
