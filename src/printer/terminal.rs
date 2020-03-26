use crate::doc;
use crate::doc::ts_type::TsTypeDefKind;
use crate::doc::DocNodeKind;

#[derive(Debug)]
pub struct TerminalPrinter {}

impl TerminalPrinter {
  pub fn new() -> TerminalPrinter {
    TerminalPrinter {}
  }

  pub fn print(&self, doc_nodes: Vec<doc::DocNode>) {
    self.print_(doc_nodes, 0);
  }

  pub fn print_details(&self, node: doc::DocNode) {
    println!(
      "Defined in {}:{}:{}.\n",
      node.location.filename, node.location.line, node.location.col
    );

    let js_doc = node.js_doc.clone();
    match node.kind {
      DocNodeKind::Function => self.print_function_signature(node, 0),
      DocNodeKind::Variable => self.print_variable_signature(node, 0),
      DocNodeKind::Class => self.print_class_signature(node, 0),
      DocNodeKind::Enum => self.print_enum_signature(node, 0),
      DocNodeKind::Interface => self.print_interface_signature(node, 0),
      DocNodeKind::TypeAlias => self.print_type_alias_signature(node, 0),
      DocNodeKind::Namespace => self.print_namespace_signature(node, 0),
    }

    if js_doc.is_some() {
      self.print_jsdoc(js_doc.unwrap(), false, 1);
    }
  }

  fn kind_order(&self, kind: &doc::DocNodeKind) -> i64 {
    match kind {
      DocNodeKind::Function => 0,
      DocNodeKind::Variable => 1,
      DocNodeKind::Class => 2,
      DocNodeKind::Enum => 3,
      DocNodeKind::Interface => 4,
      DocNodeKind::TypeAlias => 5,
      DocNodeKind::Namespace => 6,
    }
  }

  fn print_(&self, doc_nodes: Vec<doc::DocNode>, indent: i64) {
    let mut sorted = doc_nodes.clone();
    sorted.sort_unstable_by(|a, b| {
      let kind_cmp = self.kind_order(&a.kind).cmp(&self.kind_order(&b.kind));
      if kind_cmp == core::cmp::Ordering::Equal {
        a.name.cmp(&b.name)
      } else {
        kind_cmp
      }
    });

    for node in sorted {
      let kind = node.kind.clone();
      let js_doc = node.js_doc.clone();
      let namespace_def = node.namespace_def.clone();
      match kind {
        DocNodeKind::Function => self.print_function_signature(node, indent),
        DocNodeKind::Variable => self.print_variable_signature(node, indent),
        DocNodeKind::Class => self.print_class_signature(node, indent),
        DocNodeKind::Enum => self.print_enum_signature(node, indent),
        DocNodeKind::Interface => self.print_interface_signature(node, indent),
        DocNodeKind::TypeAlias => self.print_type_alias_signature(node, indent),
        DocNodeKind::Namespace => self.print_namespace_signature(node, indent),
      };
      if js_doc.is_some() {
        self.print_jsdoc(js_doc.unwrap(), true, indent);
      }
      println!("");
      match kind {
        DocNodeKind::Namespace => {
          self.print_(namespace_def.unwrap().elements, indent + 1);
          println!("");
        }
        _ => {}
      };
    }
  }

  fn render_params(&self, params: Vec<doc::ParamDef>) -> String {
    let mut rendered = String::from("");
    if params.len() > 0 {
      for param in params {
        rendered.push_str(param.name.as_str());
        if param.ts_type.is_some() {
          rendered.push_str(": ");
          rendered
            .push_str(self.render_ts_type(param.ts_type.unwrap()).as_str());
        }
        rendered.push_str(", ");
      }
      rendered.truncate(rendered.len() - 2);
    }
    rendered
  }

  fn render_ts_type(&self, ts_type: doc::ts_type::TsTypeDef) -> String {
    let kind = ts_type.kind.unwrap();
    match kind {
      TsTypeDefKind::Array => {
        format!("{}[]", self.render_ts_type(*ts_type.array.unwrap()))
      }
      TsTypeDefKind::Conditional => {
        let conditional = ts_type.conditional_type.unwrap();
        format!(
          "{} extends {} ? {} : {}",
          self.render_ts_type(*conditional.check_type),
          self.render_ts_type(*conditional.extends_type),
          self.render_ts_type(*conditional.true_type),
          self.render_ts_type(*conditional.false_type)
        )
      }
      TsTypeDefKind::FnOrConstructor => {
        let fn_or_constructor = ts_type.fn_or_constructor.unwrap();
        format!(
          "{}({}) => {}",
          if fn_or_constructor.constructor {
            "new "
          } else {
            ""
          },
          self.render_params(fn_or_constructor.params),
          self.render_ts_type(fn_or_constructor.ts_type),
        )
      }
      TsTypeDefKind::IndexedAccess => {
        let indexed_access = ts_type.indexed_access.unwrap();
        format!(
          "{}[{}]",
          self.render_ts_type(*indexed_access.obj_type),
          self.render_ts_type(*indexed_access.index_type)
        )
      }
      TsTypeDefKind::Intersection => {
        let intersection = ts_type.intersection.unwrap();
        let mut output = "".to_string();
        if intersection.len() > 0 {
          for ts_type in intersection {
            output.push_str(self.render_ts_type(ts_type).as_str());
            output.push_str(" & ")
          }
          output.truncate(output.len() - 3);
        }
        output
      }
      TsTypeDefKind::Keyword => ts_type.keyword.unwrap(),
      TsTypeDefKind::Literal => {
        let literal = ts_type.literal.unwrap();
        match literal.kind {
          doc::ts_type::LiteralDefKind::Boolean => {
            format!("{}", literal.boolean.unwrap())
          }
          doc::ts_type::LiteralDefKind::String => literal.string.unwrap(),
          doc::ts_type::LiteralDefKind::Number => {
            format!("{}", literal.number.unwrap())
          }
        }
      }
      TsTypeDefKind::Optional => "_optional_".to_string(),
      TsTypeDefKind::Parenthesized => {
        format!("({})", self.render_ts_type(*ts_type.parenthesized.unwrap()))
      }
      TsTypeDefKind::Rest => {
        format!("...{}", self.render_ts_type(*ts_type.rest.unwrap()))
      }
      TsTypeDefKind::This => "this".to_string(),
      TsTypeDefKind::Tuple => {
        let tuple = ts_type.tuple.unwrap();
        let mut output = "".to_string();
        if tuple.len() > 0 {
          for ts_type in tuple {
            output.push_str(self.render_ts_type(ts_type).as_str());
            output.push_str(", ")
          }
          output.truncate(output.len() - 2);
        }
        output
      }
      TsTypeDefKind::TypeLiteral => ts_type.repr,
      TsTypeDefKind::TypeOperator => {
        let operator = ts_type.type_operator.unwrap();
        format!(
          "{} {}",
          operator.operator,
          self.render_ts_type(operator.ts_type)
        )
      }
      TsTypeDefKind::TypeQuery => {
        format!("typeof {}", ts_type.type_query.unwrap())
      }
      TsTypeDefKind::TypeRef => {
        let type_ref = ts_type.type_ref.unwrap();
        let mut final_output = type_ref.type_name;
        if type_ref.type_params.is_some() {
          let mut output = "".to_string();
          let type_params = type_ref.type_params.unwrap();
          if type_params.len() > 0 {
            for ts_type in type_params {
              output.push_str(self.render_ts_type(ts_type).as_str());
              output.push_str(", ")
            }
            output.truncate(output.len() - 2);
          }
          final_output.push_str(format!("<{}>", output).as_str());
        }
        final_output
      }
      TsTypeDefKind::Union => {
        let union = ts_type.union.unwrap();
        let mut output = "".to_string();
        if union.len() > 0 {
          for ts_type in union {
            output.push_str(self.render_ts_type(ts_type).as_str());
            output.push_str(" | ")
          }
          output.truncate(output.len() - 3);
        }
        output
      }
    }
  }

  fn print_indent(&self, indent: i64) {
    for _ in 0..indent {
      print!("  ")
    }
  }

  fn print_jsdoc(&self, jsdoc: String, truncated: bool, indent: i64) {
    let mut lines = jsdoc.split("\n\n").map(|line| line.replace("\n", " "));
    if truncated {
      let first_line = lines.next().unwrap_or("".to_string());
      self.print_indent(indent + 1);
      println!("{}", first_line)
    } else {
      for line in lines {
        self.print_indent(indent + 1);
        println!("{}", line)
      }
    }
  }

  fn print_function_signature(&self, node: doc::DocNode, indent: i64) {
    self.print_indent(indent);
    let function_def = node.function_def.unwrap();
    let return_type = function_def.return_type.unwrap();
    println!(
      "function {}({}): {}",
      node.name,
      self.render_params(function_def.params),
      self.render_ts_type(return_type).as_str()
    );
  }

  fn print_class_signature(&self, node: doc::DocNode, indent: i64) {
    self.print_indent(indent);
    println!("class {}", node.name);
  }

  fn print_variable_signature(&self, node: doc::DocNode, indent: i64) {
    self.print_indent(indent);
    let variable_def = node.variable_def.unwrap();
    println!(
      "{} {}{}",
      match variable_def.kind {
        swc_ecma_ast::VarDeclKind::Const => "const".to_string(),
        swc_ecma_ast::VarDeclKind::Let => "let".to_string(),
        swc_ecma_ast::VarDeclKind::Var => "var".to_string(),
      },
      node.name,
      if variable_def.ts_type.is_some() {
        format!(": {}", self.render_ts_type(variable_def.ts_type.unwrap()))
      } else {
        "".to_string()
      }
    );
  }

  fn print_enum_signature(&self, node: doc::DocNode, indent: i64) {
    self.print_indent(indent);
    println!("enum {}", node.name);
  }

  fn print_interface_signature(&self, node: doc::DocNode, indent: i64) {
    self.print_indent(indent);
    println!("interface {}", node.name);
  }

  fn print_type_alias_signature(&self, node: doc::DocNode, indent: i64) {
    self.print_indent(indent);
    println!("type {}", node.name);
  }

  fn print_namespace_signature(&self, node: doc::DocNode, indent: i64) {
    self.print_indent(indent);
    println!("namespace {}", node.name);
  }
}