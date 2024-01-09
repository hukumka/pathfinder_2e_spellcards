use std::collections::HashMap;
use std::fmt::Write;
use std::io::stdin;
use tera::{from_value, Context, Result, Tera, Value};

fn from_comma_separated(value: Value) -> Result<Vec<String>> {
    let items = if let Ok(items) = from_value::<Vec<String>>(value.clone()) {
        items
    } else {
        from_value::<String>(value)?
            .split(',')
            .map(|x| x.trim().to_string())
            .collect::<Vec<_>>()
    };
    Ok(items)
}

fn traits(args: &HashMap<String, Value>) -> Result<Value> {
    // Extract 'traits' variable and parse it as `Vec<String>`
    args.get("traits")
        .cloned()
        .ok_or_else(|| "Variable traits undefined".into())
        .and_then(from_comma_separated)
        .map(|traits| {
            // then succesful construct trait boxes
            let result = traits
                .iter()
                .map(|trait_| format!("<span class=\"trait\">{}</span>", &trait_))
                .collect::<Vec<_>>()
                .join("");
            Value::String(result)
        })
}

fn get_string(args: &HashMap<String, Value>, name: &str) -> Result<String> {
    args.get(name)
        .ok_or_else(|| format!("Parameter is {} missing", name).into())
        .and_then(|x| {
            if let Value::String(result) = x {
                Ok(result.clone())
            } else {
                Err("Name must be string".into())
            }
        })
}

fn properties(args: &HashMap<String, Value>) -> Result<Value> {
    // Extract 'traits' variable and parse it as `Vec<String>`
    let name = get_string(args, "name")?;
    let value = get_string(args, "value")?;
    Ok(Value::String(format!(
        "<p><span class=\"caption\">{}</span> {}</p>",
        name, value,
    )))
}

fn success_table(args: &HashMap<String, Value>) -> Result<Value> {
    let items = ["crit_success", "success", "failure", "crit_failure"];
    let captions = ["Critical Success", "Success", "Failure", "Critical Failure"];
    let mut result = String::new();
    for (item, caption) in items.iter().zip(captions) {
        if args.contains_key(*item) {
            let value = get_string(args, item)?;
            write!(
                &mut result,
                "<div><span class=\"caption\">{}</span> {}</div>",
                caption, value
            )
            .unwrap();
        }
    }
    Ok(Value::String(result))
}

fn context() -> Context {
    let mut context = Context::new();
    context.insert("action", "<span class=\"action-icon\">1</span>");
    context.insert("two_actions", "<span class=\"action-icon\">2</span>");
    context.insert("three_actions", "<span class=\"action-icon\">3</span>");
    context.insert("free_action", "<span class=\"action-icon\">4</span>");
    context.insert("reaction", "<span class=\"action-icon\">5</span>");
    context
}

fn main() -> tera::Result<()> {
    let pages: Vec<_> = stdin()
        .lines()
        .map(|line| line.unwrap().to_string())
        .collect();
    let mut tera = Tera::new("templates/**/*.html")?;
    tera.autoescape_on(vec![]);
    tera.register_function("traits", traits);
    tera.register_function("property", properties);
    tera.register_function("success_table", success_table);
    let context = context();

    let pages = pages
        .iter()
        .map(|template| tera.render(template, &context))
        .collect::<tera::Result<Vec<_>>>()?;

    let mut context = Context::new();
    context.insert("pages", &pages);
    let result = tera.render("page.html", &context)?;
    println!("{}", result);
    Ok(())
}
