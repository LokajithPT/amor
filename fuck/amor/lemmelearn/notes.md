###                                 explaining the code 

---
##                                  import statements 
---
* **serde** : with 
    - **serialize** 
    - **deserialize** 
    - **debug** 
    - **clone**

* **env** -> just to read the **env** of **groq** 
---
##                                  structs (**all of em**)
---


`struct one ` : -> **Message** 
* we have to derive something like
    - *shit* :  **serialize** , **deserialize** , **debug** , **clone** 
* args 
    - role : String 
    - content : String 
--- 
`struct two ` : -> **ChatRequest** 
* we have to derive soemthing like 
    - shit : **serialize** 
* args 
    - model : String 
    - messages : Vec<`Message`>
--- 
`struct three ` : -> **Choice**
* we have to derive soemthing like 
    - shit : **deserialize** , **debug**
* args 
    - message : String 
---
`struct four ` : -> **ChatResponse**
* we have to derive soemthing like 
    - shit : **deserialize** 
* args 
    - choices : Vec<`Choice`>

---

---
##                                  main function 
---

# api - key -> groq api key 

# var declaration 
* `client ` -> make a new **object** 
* `client_hist` : **vector of message struct** -> new vector 
* `model` -> just the model 

# loop 

> the input 
```rust
        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() || input == "exit" {
            println!("Bye!");
            break;
        }
```

> the chat history and stuff 
```rust 

        chat_hist.push(Message {
            role: "user".to_string(),
            content: input.to_string(),
        });

        // 2. Build the payload
        let payload = ChatRequest {
            model: model.clone(),
            messages: chat_hist.clone(),
            temperature: 0.7,
        };


```

>  response variable  to send a shit 
```rust 
        let response = client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

```

> error handling or something 

```rust 
        if response.status().is_success() {
            let res_body: ChatResponse = response.json().await?;
            if let Some(choice) = res_body.choices.get(0) {
                let bot_msg = &choice.message;
                println!("\n{}\n", bot_msg.content);
                chat_hist.push(bot_msg.clone());
            }
        } else {
            // This is the "Error: $(echo "$response" | jq...)" part of your bash script
            let err_text = response.text().await?;
            println!("Error from API: {}", err_text);
        }

```












