// mcp-ui: GUI application for MCP - Model Context Protocol
//
// A Xilem UI for the Model Context Protocol

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use winit::dpi::LogicalSize;
use winit::error::EventLoopError;
use winit::window::Window;
use xilem::core::fork;
use xilem::core::one_of::{Either, OneOf3};
use xilem::view::{
    button, flex, portal, prose, sized_box, spinner, textbox, worker_raw, Axis, FlexExt,
    FlexSpacer, MainAxisAlignment, Padding,
};
use xilem::{palette, EventLoop, EventLoopBuilder, TextAlignment, WidgetView, Xilem};

// Context Response DTO
#[derive(Debug, Clone, Deserialize, Serialize)]
struct ContextResponse {
    id: Uuid,
    content: String,
    source: Option<String>,
    content_type: Option<String>,
    tags: Vec<String>,
    metadata: HashMap<String, String>,
    created_at: String,
    expires_at: Option<String>,
}

// Request to create a context
#[derive(Debug, Clone, Serialize, PartialEq)]
struct CreateContextRequest {
    content: String,
    source: Option<String>,
    content_type: Option<String>,
    tags: Vec<String>,
}

// API call result enum
#[derive(Debug)]
enum ApiResult<T> {
    Success(T),
    Error(String),
}

// Types of API requests
#[derive(Debug, PartialEq, Clone)]
enum ApiRequest {
    LoadContexts,
    CreateContext(CreateContextRequest),
    DeleteContext(Uuid),
}

// Component to represent a single context in the list
struct ContextListItem {
    context: ContextResponse,
    is_selected: bool,
}

impl ContextListItem {
    fn view(&self) -> impl WidgetView<McpApp> {
        let id = self.context.id;
        let display_content = if self.context.content.len() > 50 {
            format!("{}...", &self.context.content[..47])
        } else {
            self.context.content.clone()
        };

        button(display_content, move |state: &mut McpApp| {
            state.selected_context_id = Some(id);
        })
        //.padding(Padding::all(8.))
        //.rounded(4.)
        //.background(if self.is_selected {
        //    palette::css::DARK_SLATE_GRAY
        //} else {
        //    palette::css::SLATE_GRAY
        //})
    }
}

// Component for context details
struct ContextDetailsView {
    context: ContextResponse,
}

impl ContextDetailsView {
    fn view(&self) -> impl WidgetView<McpApp> {
        let context = &self.context;
        let id = context.id;

        // Create header section
        let header = flex((
            prose("Context Details").text_size(18.),
            FlexSpacer::Fixed(16.),
        ));

        // Create metadata section
        let metadata = flex((
            prose(format!("ID: {}", context.id)),
            FlexSpacer::Fixed(8.),
            prose(format!("Created: {}", context.created_at)),
            FlexSpacer::Fixed(8.),
        ));

        // Create content section
        let content_section = flex((
            prose("Content:"),
            sized_box(prose(&*context.content))
                .padding(Padding::all(8.))
                .rounded(4.)
                .background(palette::css::SLATE_GRAY),
            FlexSpacer::Fixed(8.),
        ));

        // Create source section
        let source_section = flex((
            prose("Source:"),
            prose(context.source.as_deref().unwrap_or("None")),
            FlexSpacer::Fixed(8.),
        ));

        // Create tags section
        let tags_section = flex((
            prose("Tags:"),
            prose(if context.tags.is_empty() {
                "None".to_string()
            } else {
                context.tags.join(", ")
            }),
            FlexSpacer::Fixed(16.),
        ));

        // Create button section
        let button_section = button("Delete Context".to_string(), move |state: &mut McpApp| {
            state.loading_operation = Some("deleting_context".to_string());
            state.delete_context_id = Some(id);
        });

        // Combine all sections
        flex((
            header,
            metadata,
            content_section,
            source_section,
            tags_section,
            button_section,
        ))
        .main_axis_alignment(MainAxisAlignment::Start)
    }
}

// Component for context creation form
struct CreateContextForm {
    content: String,
    source: String,
    tags: String,
    is_creating: bool,
}

impl CreateContextForm {
    fn view(&mut self) -> impl WidgetView<McpApp> {
        // Create the header and form sections separately
        let header = flex((
            prose("Create New Context").text_size(18.),
            FlexSpacer::Fixed(16.),
        ));

        // Create the content form section
        let content_section = self.create_form_section(
            "Content:",
            &self.content,
            |state: &mut McpApp, new_value| {
                state.new_context_content = new_value;
            },
        );

        // Create the source form section
        let source_section =
            self.create_form_section("Source:", &self.source, |state: &mut McpApp, new_value| {
                state.new_context_source = new_value;
            });

        // Create the tags form section
        let tags_section = self.create_form_section(
            "Tags (comma-separated):",
            &self.tags,
            |state: &mut McpApp, new_value| {
                state.new_context_tags = new_value;
            },
        );

        // Create the button section
        let button_section = flex((
            FlexSpacer::Fixed(16.),
            if self.is_creating {
                Either::A(spinner())
            } else {
                Either::B(button(
                    "Create Context".to_string(),
                    |state: &mut McpApp| {
                        if state.new_context_content.is_empty() {
                            state.status_message = "Content cannot be empty".to_string();
                            return;
                        }
                        state.loading_operation = Some("creating_context".to_string());
                    },
                ))
            },
        ));

        // Use tuple instead of vector for flex
        flex((
            header,
            content_section,
            source_section,
            tags_section,
            button_section,
        ))
        .main_axis_alignment(MainAxisAlignment::Start)
    }

    fn create_form_section<F>(
        &self,
        label: &str,
        value: &str,
        update_fn: F,
    ) -> impl WidgetView<McpApp>
    where
        F: Fn(&mut McpApp, String) + Send + Sync + 'static,
    {
        flex((
            prose(label),
            FlexSpacer::Fixed(4.),
            textbox(value.to_string(), update_fn),
            FlexSpacer::Fixed(8.),
        ))
    }
}

// Main app state
struct McpApp {
    contexts: Vec<ContextResponse>,
    status_message: String,
    new_context_content: String,
    new_context_source: String,
    new_context_tags: String,
    selected_context_id: Option<Uuid>,
    loading_operation: Option<String>,
    delete_context_id: Option<Uuid>,
    api_url: String,
}

impl Default for McpApp {
    fn default() -> Self {
        Self {
            contexts: Vec::new(),
            status_message: "Ready".to_string(),
            new_context_content: String::new(),
            new_context_source: String::new(),
            new_context_tags: String::new(),
            selected_context_id: None,
            loading_operation: None,
            delete_context_id: None,
            api_url: "http://localhost:3000".to_string(),
        }
    }
}

impl McpApp {
    fn view(&mut self) -> impl WidgetView<Self> {
        // Create the header
        let header = self.create_header();

        // Create the sidebar with contexts list
        let sidebar = self.create_sidebar();

        // Create the main content area
        let main_content = self.create_main_content();

        // Capture API request into a local variable to track changes
        let api_request = self.get_api_request();

        // Add debug output to help diagnose issues
        if let Some(req) = &api_request {
            println!("API Request: {:?}", req);
        }

        // Combine the layout
        let content = flex((
            header,
            flex((
                sidebar,
                sized_box(portal(main_content))
                    .padding(Padding::all(16.))
                    .flex(1.),
            ))
            .direction(Axis::Horizontal)
            .flex(1.),
        ));

        // Store API URL in a local variable for consistent usage
        let api_url = self.api_url.clone();

        // Add API worker that responds to api_request changes
        fork(
            content,
            worker_raw(
                api_request,
                move |proxy, mut rx| {
                    let api_base_url = api_url.clone();
                    async move {
                        while let Some(request) = rx.recv().await {
                            let proxy = proxy.clone();
                            let base_url = api_base_url.clone();

                            // Only proceed if we have a request
                            if let Some(request) = request {
                                println!("Worker received request: {:?}", request);

                                tokio::task::spawn(async move {
                                    let result = match request {
                                        ApiRequest::LoadContexts => fetch_contexts(&base_url).await,
                                        ApiRequest::CreateContext(req) => {
                                            create_context(&base_url, req).await
                                        }
                                        ApiRequest::DeleteContext(id) => {
                                            delete_context(&base_url, id).await
                                        }
                                    };
                                    println!("API call completed: {:?}", result);
                                    drop(proxy.message(result));
                                });
                            }
                        }
                    }
                },
                |state: &mut Self, result| {
                    println!("Handling API result");
                    match result {
                        ApiResult::Success(contexts) => {
                            state.contexts = contexts;
                            state.status_message =
                                format!("{} contexts loaded", state.contexts.len());
                        }
                        ApiResult::Error(error) => {
                            state.status_message = error;
                        }
                    }

                    // Reset form if we were creating a context
                    if state.loading_operation == Some("creating_context".into()) {
                        state.new_context_content = String::new();
                        state.new_context_source = String::new();
                        state.new_context_tags = String::new();
                    }

                    // Clear the delete context ID if we were deleting
                    state.delete_context_id = None;

                    // Always clear the loading operation
                    state.loading_operation = None;
                },
            ),
        )
    }

    fn create_header(&self) -> impl WidgetView<Self> {
        flex((
            prose("MCP - Model Context Protocol")
                .text_size(20.)
                .brush(palette::css::WHITE),
            FlexSpacer::Flex(1.),
            prose(format!("Status: {}", self.status_message))
                .text_size(14.)
                .brush(palette::css::WHITE),
        ))
        .direction(Axis::Horizontal)
        //.background(palette::css::DARK_SLATE_GRAY)
        //.padding(Padding::all(8.))
    }

    fn create_sidebar(&self) -> impl WidgetView<Self> {
        let is_loading = self.loading_operation == Some("loading_contexts".into());

        // Create header section
        let header = flex((
            prose("Contexts").text_size(18.),
            FlexSpacer::Flex(1.),
            if is_loading {
                Either::A(spinner())
            } else {
                Either::B(prose(format!("{} contexts", self.contexts.len())))
            },
        ))
        .direction(Axis::Horizontal);

        // Create list section
        let contexts_list = flex(
            self.contexts
                .iter()
                .map(|context| {
                    let is_selected = self.selected_context_id == Some(context.id);
                    let item = ContextListItem {
                        context: context.clone(),
                        is_selected,
                    };
                    item.view()
                })
                .collect::<Vec<_>>(),
        );
        //.spacing(4.);

        // Create button section
        let button_section = button("Refresh Contexts".to_string(), |state: &mut McpApp| {
            state.loading_operation = Some("loading_contexts".to_string());
        });

        // Combine all sections
        sized_box(portal(flex((
            header,
            FlexSpacer::Fixed(8.),
            contexts_list,
            FlexSpacer::Fixed(16.),
            button_section,
        ))))
        .width(300.)
        .padding(Padding::all(16.))
        .background(palette::css::SLATE_GRAY)
    }

    fn create_main_content(&mut self) -> impl WidgetView<Self> {
        if let Some(selected_id) = self.selected_context_id {
            if let Some(context) = self.contexts.iter().find(|c| c.id == selected_id) {
                // Show selected context details
                let details = ContextDetailsView {
                    context: context.clone(),
                };
                OneOf3::A(details.view())
            } else {
                OneOf3::B(prose("Context not found").alignment(TextAlignment::Middle))
            }
        } else {
            // Show context creation form
            let is_creating = self.loading_operation == Some("creating_context".into());
            let mut form = CreateContextForm {
                content: self.new_context_content.clone(),
                source: self.new_context_source.clone(),
                tags: self.new_context_tags.clone(),
                is_creating,
            };
            OneOf3::C(form.view())
        }
    }

    // Generate appropriate API request based on current state
    fn get_api_request(&self) -> Option<ApiRequest> {
        match self.loading_operation.as_deref() {
            Some("loading_contexts") => {
                println!("Creating LoadContexts request");
                Some(ApiRequest::LoadContexts)
            }
            Some("creating_context") => {
                println!("Creating CreateContext request");
                // Parse tags
                let tags = self
                    .new_context_tags
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                let request = CreateContextRequest {
                    content: self.new_context_content.clone(),
                    source: if self.new_context_source.is_empty() {
                        None
                    } else {
                        Some(self.new_context_source.clone())
                    },
                    content_type: None,
                    tags,
                };

                Some(ApiRequest::CreateContext(request))
            }
            Some("deleting_context") => {
                if let Some(id) = self.delete_context_id {
                    println!("Creating DeleteContext request for ID: {}", id);
                    Some(ApiRequest::DeleteContext(id))
                } else {
                    println!("Missing delete_context_id");
                    None
                }
            }
            _ => {
                // No active operation
                None
            }
        }
    }
}

// API functions

async fn fetch_contexts(base_url: &str) -> ApiResult<Vec<ContextResponse>> {
    println!("Fetching contexts from: {}/contexts", base_url);
    let client = reqwest::Client::new();
    match client
        .get(&format!("{}/contexts?limit=50", base_url))
        .send()
        .await
    {
        Ok(response) => {
            println!("Response status: {}", response.status());
            if response.status().is_success() {
                match response.json::<Vec<ContextResponse>>().await {
                    Ok(contexts) => {
                        println!("Received {} contexts", contexts.len());
                        ApiResult::Success(contexts)
                    }
                    Err(e) => {
                        println!("Error parsing contexts: {}", e);
                        ApiResult::Error(format!("Failed to parse contexts: {}", e))
                    }
                }
            } else {
                let error_msg = format!("Failed to load contexts: HTTP {}", response.status());
                println!("{}", error_msg);
                ApiResult::Error(error_msg)
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to load contexts: {}", e);
            println!("{}", error_msg);
            ApiResult::Error(error_msg)
        }
    }
}

async fn create_context(
    base_url: &str,
    request: CreateContextRequest,
) -> ApiResult<Vec<ContextResponse>> {
    println!("Creating context at: {}/contexts", base_url);
    println!("Request: {:?}", request);

    let client = reqwest::Client::new();

    match client
        .post(&format!("{}/contexts", base_url))
        .json(&request)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            println!("Response status: {}", status);
            if status.is_success() {
                println!("Context created successfully");
                // After successfully creating a context, reload all contexts
                fetch_contexts(base_url).await
            } else {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                let error_msg =
                    format!("Failed to create context: HTTP {} - {}", status, error_text);
                println!("{}", error_msg);
                ApiResult::Error(error_msg)
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to create context: {}", e);
            println!("{}", error_msg);
            ApiResult::Error(error_msg)
        }
    }
}

async fn delete_context(base_url: &str, id: Uuid) -> ApiResult<Vec<ContextResponse>> {
    let client = reqwest::Client::new();

    match client
        .delete(&format!("{}/contexts/{}", base_url, id))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                // After successfully deleting a context, reload all contexts
                fetch_contexts(base_url).await
            } else {
                ApiResult::Error(format!(
                    "Failed to delete context: HTTP {}",
                    response.status()
                ))
            }
        }
        Err(e) => ApiResult::Error(format!("Failed to delete context: {}", e)),
    }
}

fn run(event_loop: EventLoopBuilder) -> Result<(), EventLoopError> {
    let app = Xilem::new(McpApp::default(), McpApp::view);
    let min_window_size = LogicalSize::new(800., 600.);

    let window_attributes = Window::default_attributes()
        .with_title("MCP - Model Context Protocol")
        .with_resizable(true)
        .with_min_inner_size(min_window_size);

    app.run_windowed_in(event_loop, window_attributes)
}

fn main() -> Result<(), EventLoopError> {
    run(EventLoop::with_user_event())
}

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;

    let mut event_loop = EventLoop::with_user_event();
    event_loop.with_android_app(app);

    run(event_loop).expect("Can create app");
}
