use rbx_dom_weak::{types::{Ref, UniqueId, Variant}, WeakDom};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use ustr::ustr;

#[derive(Debug, Clone)]
struct UICorner {
    bottom_left: Option<Variant>,
    bottom_right: Option<Variant>,
    top_left: Option<Variant>,
    top_right: Option<Variant>,
}

fn collect(dom: &WeakDom, referent: Ref, corners: &mut HashMap<UniqueId, UICorner>) {
    if let Some(instance) = dom.get_by_ref(referent) {
        if instance.class == "UICorner" {
            if let Some(Variant::UniqueId(uid)) = instance.properties.get(&ustr("UniqueId")) {
                corners.insert(
                    *uid,
                    UICorner {
                        bottom_left: instance.properties.get(&ustr("BottomLeftRadius")).cloned(),
                        bottom_right: instance.properties.get(&ustr("BottomRightRadius")).cloned(),
                        top_left: instance.properties.get(&ustr("TopLeftRadius")).cloned(),
                        top_right: instance.properties.get(&ustr("TopRightRadius")).cloned(),
                    },
                );
            }
        }

        for &child in instance.children() {
            collect(dom, child, corners);
        }
    }
}

fn apply(dom: &mut WeakDom, referent: Ref, corners: &HashMap<UniqueId, UICorner>) {
    let children = if let Some(inst) = dom.get_by_ref(referent) {
        inst.children().to_vec()
    } else {
        vec![]
    };

    if let Some(instance) = dom.get_by_ref_mut(referent) {
        if instance.class == "UICorner" {
            if let Some(Variant::UniqueId(uid)) = instance.properties.get(&ustr("UniqueId")) {
                if let Some(data) = corners.get(uid) {
                    if let Some(val) = &data.bottom_left {
                        instance.properties.insert(ustr("BottomLeftRadius"), val.clone());
                    }

                    if let Some(val) = &data.bottom_right {
                        instance.properties.insert(ustr("BottomRightRadius"), val.clone());
                    }

                    if let Some(val) = &data.top_left {
                        instance.properties.insert(ustr("TopLeftRadius"), val.clone());
                    }

                    if let Some(val) = &data.top_right {
                        instance.properties.insert(ustr("TopRightRadius"), val.clone());
                    }
                }
            }
        }
    }

    for child in children {
        apply(dom, child, corners);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Opening old.rbxl...");
    let old_dom = rbx_binary::from_reader(BufReader::new(File::open("old.rbxl")?))?;

    println!("Opening new.rbxl...");
    let mut new_dom = rbx_binary::from_reader(BufReader::new(File::open("new.rbxl")?))?;

    println!("Working...");
    let mut corners = HashMap::new();

    let old_root = old_dom.root_ref();
    collect(&old_dom, old_root, &mut corners);

    let new_root = new_dom.root_ref();
    apply(&mut new_dom, new_root, &corners);

    println!("Saving changes...");
    let out_file = BufWriter::new(File::create("new.rbxl")?);
    rbx_binary::to_writer(out_file, &new_dom, new_dom.root().children())?;

    println!("Done!");
    std::process::exit(0);
}