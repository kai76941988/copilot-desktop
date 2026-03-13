use serde_json::json;

pub fn get_prompts() -> Vec<serde_json::Value> {
    vec![
        json!({
            "name": "analyze_page_structure",
            "description": "Analyze the structure of the current page",
            "arguments": []
        }),
        json!({
            "name": "extract_main_content",
            "description": "Extract the main content from the current page",
            "arguments": []
        }),
        json!({
            "name": "find_interactive_elements",
            "description": "Find all interactive elements (buttons, links, inputs) on the page",
            "arguments": []
        }),
        json!({
            "name": "summarize_page",
            "description": "Generate a summary of the current page",
            "arguments": []
        }),
        json!({
            "name": "check_login_status",
            "description": "Check if the user is logged in to the current site",
            "arguments": []
        }),
        json!({
            "name": "navigate_and_wait",
            "description": "Navigate to a URL and wait for specific content to load",
            "arguments": [
                {
                    "name": "url",
                    "description": "The URL to navigate to",
                    "required": true
                },
                {
                    "name": "waitSelector",
                    "description": "CSS selector to wait for",
                    "required": false
                }
            ]
        }),
        json!({
            "name": "fill_form",
            "description": "Fill a form with provided data",
            "arguments": [
                {
                    "name": "formData",
                    "description": "Object mapping CSS selectors to values",
                    "required": true
                }
            ]
        }),
        json!({
            "name": "scroll_and_extract",
            "description": "Scroll through the page and extract all content",
            "arguments": [
                {
                    "name": "selector",
                    "description": "CSS selector for items to extract",
                    "required": true
                }
            ]
        }),
    ]
}
