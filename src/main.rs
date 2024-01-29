use serde::{Deserialize, Serialize};
use std::{
    io,
    process::{Command, Stdio},
};

#[derive(Serialize, Deserialize, Debug)]
struct Window {
    uid: u32,
    pid: u32,
    space: u32,
    title: String,
    app: String,
}

#[derive(Serialize, Debug)]
struct Item {
    uid: u32,
    title: String,
    subtitle: String,
    icon: Icon,
    arg: [String; 2],
}

impl From<Window> for Item {
    fn from(win: Window) -> Self {
        Self {
            uid: win.uid,
            title: win.title,
            subtitle: win.app,
            icon: Icon::default(),
            arg: [win.space.to_string(), win.uid.to_string()],
        }
    }
}

#[derive(Default, Serialize, Debug)]
struct Icon {
    path: String,
    r#type: String,
}

#[derive(Serialize, Debug)]
struct Response {
    items: Vec<Item>,
}

fn main() {
    // Alfred passes in a single argument for the user query.
    let query = std::env::args().nth(1);

    let yabai_info = Command::new("/usr/local/bin/yabai")
        .args(["-m", "query", "--windows"])
        .stdout(Stdio::piped())
        .spawn();

    let jq_info = Command::new("jq")
        .arg("[.[] | {uid: .id, pid, space, title, app}]")
        .stdin(Stdio::from(yabai_info.unwrap().stdout.unwrap()))
        .output()
        .expect("Get windows info failed");
    let output = &String::from_utf8_lossy(&jq_info.stdout);

    // Convert result to Window struct list.
    let mut windows: Vec<Window> = serde_json::from_str(output).unwrap();
    let mut items = vec![];
    // filter by query
    if let Some(query) = query {
        let arg = query.to_lowercase();
        windows.retain(|win| {
            win.app.to_lowercase().contains(&arg) || win.title.to_lowercase().contains(&arg)
        });
    }

    let re = regex::Regex::new(".*?.app").unwrap();
    for win in windows.into_iter() {
        let pid = win.pid;
        let output = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "command"])
            .output()
            .expect("can not get process info. pid: {pid}");
        let out = String::from_utf8(output.stdout).unwrap();
        let path = re.captures(&out).unwrap().get(0).unwrap().as_str();

        let mut item = Item::from(win);
        item.icon = Icon {
            path: path.to_owned(),
            r#type: String::from("fileicon"),
        };
        items.push(item);
    }

    // Output to Alfred!
    serde_json::to_writer(io::stdout(), &Response { items }).unwrap();
}
