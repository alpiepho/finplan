use rmcp::{
    ErrorData as McpError, ServerHandler,
    model::*,
    service::{RequestContext, RoleServer},
};

use crate::{
    prompts, resources,
    state::{SharedState, new_shared_state},
    tools,
};

/// Main MCP server for FinPlan scenario construction.
#[derive(Clone)]
pub struct FinplanMcpServer {
    pub state: SharedState,
}

impl FinplanMcpServer {
    pub fn new() -> Self {
        Self {
            state: new_shared_state(),
        }
    }
}

impl Default for FinplanMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerHandler for FinplanMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "FinPlan MCP server for building financial planning scenario YAML files. \
                 Use resources to read the YAML schema, tools to construct scenarios \
                 step-by-step, and prompts for guided workflows."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder()
                .enable_resources()
                .enable_tools()
                .enable_prompts()
                .build(),
            server_info: Implementation {
                name: "finplan-mcp".into(),
                title: None,
                version: env!("CARGO_PKG_VERSION").into(),
                description: Some("MCP server for FinPlan scenario YAML construction".into()),
                icons: None,
                website_url: None,
            },
            ..Default::default()
        }
    }

    // ── Resources ──────────────────────────────────────────────────────

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _cx: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: resources::list_resources(),
            next_cursor: None,
            meta: None,
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _cx: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let uri = request.uri.as_str();
        match resources::read_resource(uri) {
            Some(content) => Ok(ReadResourceResult {
                contents: vec![content],
            }),
            None => Err(McpError::invalid_params(
                format!("Unknown resource URI: {}", uri),
                None,
            )),
        }
    }

    // ── Tools ──────────────────────────────────────────────────────────

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _cx: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult {
            tools: tools::list_tools(),
            next_cursor: None,
            meta: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _cx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let name = request.name.as_ref();
        let args = request.arguments.unwrap_or_default();
        tools::call_tool(name, args, &self.state).await
    }

    // ── Prompts ────────────────────────────────────────────────────────

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _cx: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult {
            prompts: prompts::list_prompts(),
            next_cursor: None,
            meta: None,
        })
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _cx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let name = request.name.as_ref();
        let args = request.arguments.unwrap_or_default();
        prompts::get_prompt(name, args)
    }
}
