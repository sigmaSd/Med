use gtk::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::rc::Rc;

type RefMap<U, T> = Rc<RefCell<HashMap<U, T>>>;

fn main() {
    gtk::init().unwrap();
    let mut med = Med::default();

    med.build_gui();
    med.wire();
    gtk::main();
}

struct Med {
    btns: HashMap<String, Button>,
    entrys: HashMap<String, Entry>,
    boxes: HashMap<String, Box>,
    wins: HashMap<String, Window>,
    database: RefMap<String, PathBuf>,
    patients_rows: RefMap<String, Box>,
}

impl Default for Med {
    fn default() -> Self {
        let database = Rc::new(RefCell::new(Self::parse_database()));
        Self {
            btns: HashMap::new(),
            entrys: HashMap::new(),
            boxes: HashMap::new(),
            wins: HashMap::new(),
            patients_rows: Rc::new(RefCell::new(HashMap::new())),
            database,
        }
    }
}

impl Med {
    fn build_gui(&mut self) {
        let add_btn = Button::new_with_label("Add");
        let rm_btn = Button::new_with_label("Rm");
        let search_bar = Entry::new();

        let hbox = Box::new(Orientation::Horizontal, 10);
        hbox.add(&add_btn);
        hbox.add(&rm_btn);

        let vbox = Box::new(Orientation::Vertical, 10);
        vbox.add(&hbox);
        vbox.add(&search_bar);
        for name in self.database.borrow().keys() {
            Self::sig_patient_add(&self.patients_rows, name.to_string(), &vbox, None, None)
        }

        let win = Window::new(WindowType::Toplevel);
        win.set_title("Med");
        win.add(&vbox);
        win.show_all();

        self.btns.insert("add".to_string(), add_btn);
        self.btns.insert("rm".to_string(), rm_btn);
        self.entrys.insert("search".to_string(), search_bar);

        self.boxes.insert("hbox".to_string(), hbox);
        self.boxes.insert("vbox".to_string(), vbox);

        self.wins.insert("Win".to_string(), win);
    }

    fn wire(&mut self) {
        self.sig_search_patient();
        self.sig_add_patients_btn();
        self.sig_remove_patient_btn();
    }

    // parse
    fn parse_database() -> HashMap<String, PathBuf> {
        let mut map_base = HashMap::new();
        let med_dir = Self::med_dir();

        if !Self::med_dir().is_dir() {
            fs::create_dir(&med_dir).unwrap();
        }

        for patient_dir in fs::read_dir(med_dir).unwrap() {
            let patient_dir = patient_dir.unwrap().path();
            let patient_name = Self::read_patient_data(&patient_dir).0.unwrap();
            map_base.insert(patient_name, patient_dir);
        }
        map_base
    }

    // signals
    fn sig_search_patient(&mut self) {
        // clone ref
        let search_bar = self.entrys["search"].clone();
        let vbox = self.boxes["vbox"].clone();
        let p_b = self.patients_rows.clone();

        search_bar.connect_property_text_length_notify(move |search_bar| {
            let search = search_bar.get_text().unwrap().to_string();
            let mut visible = vec![];
            let p_b = p_b.borrow();
            p_b.keys().for_each(|p_name| {
                if p_name.contains(&search) {
                    let p_btn = &p_b[p_name];
                    visible.push(p_btn);
                }
            });
            for c in vbox.get_children().iter().skip(2) {
                vbox.remove(c);
            }
            for v in visible {
                vbox.add(v);
            }
        });
    }

    fn sig_add_patients_btn(&mut self) {
        // clone ref
        let vbox = self.boxes["vbox"].clone();
        let patients_rows = self.patients_rows.clone();
        let database = self.database.clone();
        //
        self.btns["add"].connect_clicked(move |_btn| {
            let entry_text = Entry::new();
            let entry_win = Window::new(WindowType::Toplevel);
            entry_win.add(&entry_text);
            entry_win.set_title("Name");

            // clone ref
            let vbox = vbox.clone();
            let ew = entry_win.clone();
            let patients_rows = patients_rows.clone();
            let database = database.clone();
            //

            entry_text.connect_activate(move |et| {
                Self::sig_patient_add(
                    &patients_rows,
                    et.get_text().unwrap().to_string(),
                    &vbox,
                    Some(&database),
                    Some(&ew),
                );
            });
            entry_win.show_all();
        });
    }

    fn sig_patient_add(
        patients_rows: &RefMap<String, Box>,
        patient_name: String,
        vbox: &Box,
        database: Option<&RefMap<String, PathBuf>>,
        ew: Option<&Window>,
    ) {
        //clone
        let pname = patient_name.clone();

        let patient_hash: String = {
            let hash: [u8; 16] = md5::compute(patient_name.clone()).into();
            hash.iter().fold(0, |acc, x| acc + *x as usize).to_string()
        };

        let p_dir: std::path::PathBuf = {
            let mut med_dir = Self::med_dir();
            med_dir.push(patient_hash);
            med_dir
        };

        if !std::path::Path::exists(&p_dir) {
            fs::create_dir(&p_dir).unwrap();
        }

        if let Some(database) = database {
            database
                .borrow_mut()
                .insert(patient_name.clone(), p_dir.clone());
        }

        // if no data is present add patient name as data
        // if Self::read_patient_data(&p_dir).0.is_none() {
        //     Self::sig_save_patient_data(&p_dir, patient_name.clone(), "".to_string());
        // }

        let hbox = Self::create_patient_row(patient_name, p_dir);

        vbox.add(&hbox);
        patients_rows.borrow_mut().insert(pname, hbox);

        vbox.show_all();

        if let Some(ew) = ew {
            ew.destroy()
        };
    }
    fn sig_save_patient_data(p_dir: &PathBuf, p_name: String, p_data: String) {
        if !std::path::Path::exists(&p_dir) {
            std::fs::create_dir(&p_dir).unwrap();
        }

        let data = {
            let mut d = p_dir.clone();
            d.push("data");
            d
        };

        let mut data = std::fs::File::create(data).unwrap();
        let name_and_data = {
            let mut t = p_name.clone();
            t.push('\n');
            t.push_str(p_data.as_str());
            t
        };
        write!(data, "{}", name_and_data).unwrap();
    }
    fn sig_remove_patient_btn(&mut self) {
        // ref clone
        let big_vbox = self.boxes["vbox"].clone();
        let patients_rows = self.patients_rows.clone();
        let database = self.database.clone();

        self.btns["rm"].connect_clicked(move |_btn| {
            let entry_text = Entry::new();
            let rm_btn = Button::new_with_label("Remove");
            let vbox = Box::new(Orientation::Vertical, 10);
            let entry_win = Window::new(WindowType::Toplevel);

            vbox.pack_start(&entry_text, false, false, 10);
            vbox.pack_start(&rm_btn, false, false, 10);
            vbox.add(&entry_text);
            vbox.add(&rm_btn);

            entry_win.add(&vbox);
            entry_win.set_title("Remove Patient");
            entry_win.show_all();

            //ref clone
            let patients_rows = patients_rows.clone();
            let et = entry_text.clone();
            let ew = entry_win.clone();
            let big_vbox = big_vbox.clone();
            let database = database.clone();

            rm_btn.connect_clicked(move |_rm_btn| {
                let p_name = et.get_text().unwrap().to_string();
                let p_btn = &patients_rows.borrow()[&p_name];
                let p_dir = &database.borrow()[&p_name];

                let _ = fs::remove_dir_all(&p_dir);

                big_vbox.remove(p_btn);
                ew.destroy();
            });
        });
    }
    // gui pieces
    fn create_patient_row(patient_name: String, p_dir: PathBuf) -> Box {
        // name diag de ds nd
        let (diag, de, ds, nd) = Self::read_patient_header(&p_dir);

        // name button
        let btn = Button::new_with_label(&patient_name);

        btn.connect_clicked(move |_btn| {
            Self::create_patient_win(&patient_name, &p_dir);
        });

        // the rest
        let hbox = Box::new(Orientation::Horizontal, 10);
        hbox.add(&btn);
        for col in [diag, de, ds, nd].iter() {
            let label = Label::new(col.as_str());
            hbox.add(&label);
        }

        hbox
    }

    fn create_patient_win(patient_name: &String, p_dir: &PathBuf) {

        let save_btn = Button::new_with_label("Save");

        let header_box = Box::new(Orientation::Vertical, 10);
        let entry_text = Entry::new();

        let hbox = Box::new(Orientation::Horizontal, 10);
        hbox.pack_start(&entry_text, true, true, 10);
        hbox.pack_start(&header_box, false, false, 10);

        let (diag, de, ds, nd) = Self::read_patient_header(&p_dir);
        for (idx, header) in [diag, de, ds, nd].iter().enumerate() {
            match idx {
                0 => {
                    let label = Label::new("Diag: ");
                    let entry = Entry::new();
                    entry.set_text(&header);

                    let hbox = Box::new(Orientation::Horizontal, 5);
                    hbox.add(&label);
                    hbox.add(&entry);

                    header_box.add(&hbox);
                },
                1 => {
                    let label = Label::new("DE:  ");
                    let entry = Entry::new();
                    entry.set_text(&header);

                    let hbox = Box::new(Orientation::Horizontal, 5);
                    hbox.add(&label);
                    hbox.add(&entry);

                    header_box.add(&hbox);
                },
                2 => {
                    let label = Label::new("DS:  ");
                    let entry = Entry::new();
                    entry.set_text(&header);

                    let hbox = Box::new(Orientation::Horizontal, 5);
                    hbox.add(&label);
                    hbox.add(&entry);

                    header_box.add(&hbox);
                },
                3 => {
                    let label = Label::new("ND:  ");
                    let entry = Entry::new();
                    entry.set_text(&header);

                    let hbox = Box::new(Orientation::Horizontal, 5);
                    hbox.add(&label);
                    hbox.add(&entry);

                    header_box.add(&hbox);
                },
                _ => panic!("??")
            }

        }

        let vbox = Box::new(Orientation::Vertical, 10);

        vbox.pack_start(&hbox, true, true, 10);
        vbox.pack_start(&save_btn, false, false, 10);

        let entry_win = Window::new(WindowType::Toplevel);
        entry_win.add(&vbox);
        entry_win.set_title(&patient_name);
        entry_text.set_text(&Self::read_patient_data(&p_dir).1);

        // clone ref
        let patient_name = RefCell::new(patient_name.clone());
        let ew = entry_win.clone();
        let et = entry_text.clone();

        let p_dir = p_dir.clone();

        save_btn.connect_clicked(move |_| {
            let p_name = patient_name.borrow().to_string();
            let p_data = et.get_text().unwrap().to_string();
            Self::sig_save_patient_data(&p_dir, p_name, p_data);
            ew.destroy();
        });

        entry_win.show_all();
    }

    // manipulate patient data
    fn read_patient_data(p_dir: &PathBuf) -> (Option<String>, String) {
        let data_file = {
            let mut data = p_dir.clone();
            data.push("data");
            data
        };

        if !std::path::Path::exists(&data_file) {
            return (None, "".to_string());
        }

        let data = {
            let mut data = String::new();
            let mut d = fs::File::open(data_file).unwrap();
            d.read_to_string(&mut data).unwrap();
            data.to_string()
        };
        let data: Vec<String> = data.lines().map(|l| l.to_string()).collect();
        let patient_name = data[0].clone();
        let patient_data: String = data.into_iter().skip(1).collect();
        (Some(patient_name), patient_data)
    }

    fn read_patient_header(p_dir: &PathBuf) -> (String, String, String, String) {
        let header_file = {
            let mut d = p_dir.clone();
            d.push("header");
            d
        };

        if !std::path::Path::exists(&header_file) {
            return Default::default();
        }

        let data = {
            let mut data = String::new();
            let mut d = fs::File::open(header_file).unwrap();
            d.read_to_string(&mut data).unwrap();
            data.to_string()
        };

        let cols: Vec<String> = data
            .lines()
            .enumerate()
            .map(|(i, l)| match i {
                0 => l.split("Diag: ").nth(1).unwrap().to_string(),
                1 => l.split("DE: ").nth(1).unwrap().to_string(),
                2 => l.split("DS: ").nth(1).unwrap().to_string(),
                3 => l.split("ND: ").nth(1).unwrap().to_string(),
                _ => {
                    eprintln!("Patient header parsing Error!");
                    Default::default()
                }
            })
            .collect();

        (
            cols[0].clone(),
            cols[1].clone(),
            cols[2].clone(),
            cols[3].clone(),
        )
    }

    // helpers
    fn med_dir() -> std::path::PathBuf {
        let mut med_dir = dirs::config_dir().unwrap();
        med_dir.push("Med");
        med_dir
    }
}
