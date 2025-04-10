use crate::conv::Editor;
use crate::conv::converter::error;
use crate::conv::editor::GrepFlag;
use eframe::egui::text::LayoutJob;
use eframe::egui::{Color32, FontFamily, FontId, TextFormat, Ui, vec2};
use std::iter::zip;

pub fn regex_replace(ui: &mut Ui, editor: &mut Editor) {
    editor.output.clear();

    let flag = &editor.regex.regex_flag;
    let re = if flag.ignore {
        &format!("(?i){}", &editor.regex.re)
    } else {
        &editor.regex.re
    };
    match fancy_regex::Regex::new(re) {
        Ok(re) => match flag.single {
            true => {
                if flag.replace {
                    regex_replace_highlighter(ui, &re, &editor.code, &editor.regex.text);
                } else {
                    regex_highlighter(ui, &re, &editor.code);
                }
            }
            false => {
                let line = editor.code.split("\n").collect::<Vec<&str>>();
                line.iter().for_each(|hay| {
                    if flag.replace {
                        regex_replace_highlighter(ui, &re, hay, &editor.regex.text);
                    } else {
                        regex_highlighter(ui, &re, hay);
                    };
                });
            }
        },
        Err(e) => {
            error(ui, "warn", &e.to_string());
        }
    };
}

pub fn regex_grep(ui: &mut Ui, editor: &mut Editor) {
    editor.output.clear();

    let flag = &editor.regex.grep_flag;
    let line = editor.code.split("\n").collect::<Vec<&str>>();

    let re = if flag.ignore {
        &format!("(?i){}", &editor.regex.re)
    } else {
        &editor.regex.re
    };

    match fancy_regex::Regex::new(re) {
        Ok(re) => {
            line.into_iter()
                .for_each(|hay| grep_highlighter(ui, &re, hay, flag));
        }
        Err(e) => {
            error(ui, "warn", &e.to_string());
        }
    }
}

#[inline]
fn grep_highlighter(ui: &mut Ui, re: &fancy_regex::Regex, hay: &str, flag: &GrepFlag) {
    let mut a = re
        .find_iter(hay)
        .map(|m| m.unwrap().as_str())
        .collect::<Vec<&str>>();

    let n = a.len();
    let show = || {
        a.push("");
        let b = re.split(hay).map(|x| x.unwrap()).collect::<Vec<&str>>();
        zipper(ui, a, b);
    };

    if flag.invert {
        if n == 0 {
            show();
        }
    } else if n != 0 {
        show();
    }
}

#[inline]
fn regex_highlighter(ui: &mut Ui, re: &fancy_regex::Regex, hay: &str) {
    let mut a = re
        .find_iter(hay)
        .map(|m| m.unwrap().as_str())
        .collect::<Vec<&str>>();
    a.push("");
    let b = re.split(hay).map(|x| x.unwrap()).collect::<Vec<&str>>();
    zipper(ui, a, b);
}

#[inline]
fn regex_replace_highlighter(ui: &mut Ui, re: &fancy_regex::Regex, hay: &str, rep: &str) {
    let a = re
        .find_iter(hay)
        .map(|m| m.unwrap().as_str())
        .collect::<Vec<&str>>();
    let mut c = a.iter().map(|x| re.replace(x, rep)).collect::<Vec<_>>();
    c.push("".into());

    let b = re.split(hay).map(|x| x.unwrap()).collect::<Vec<_>>();
    zipper(ui, c, b);
}

#[inline]
fn zipper<T>(ui: &mut Ui, a: Vec<T>, b: Vec<&str>)
where
    T: AsRef<str>,
{
    ui.spacing_mut().item_spacing = vec2(0., 0.);

    let mut job = LayoutJob::default();
    zip(a, b).for_each(|(x, y)| {
        job.append(
            y,
            0.,
            TextFormat {
                font_id: FontId::new(12.5, FontFamily::Proportional),
                ..Default::default()
            },
        );
        job.append(
            x.as_ref(),
            0.,
            TextFormat {
                font_id: FontId::new(12.5, FontFamily::Proportional),
                color: Color32::ORANGE,
                ..Default::default()
            },
        );
    });

    ui.label(job);
    ui.end_row();
}
