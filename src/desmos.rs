use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct DesmosExpression {
    id: String,
    color: String,
    latex: String,
}

#[derive(Serialize, Deserialize)]
struct DesmosExpressions {
    list: Vec<DesmosExpression>,
}

#[derive(Serialize, Deserialize)]
struct DesmosViewport {
    xmin: f64,
    xmax: f64,
    ymin: f64, // useless but needed
    ymax: f64, // useless but needed
}

#[derive(Serialize, Deserialize)]
struct DesmosGraph {
    viewport: DesmosViewport,
}

#[derive(Serialize, Deserialize)]
struct DesmosState {
    version: u64, // useless but needed
    graph: DesmosGraph,
    expressions: DesmosExpressions,
}
