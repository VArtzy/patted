### **Web3 DeAI Pet and Wildlife Identifier**

#### **Description**:  
A web3 application that allows users to upload images of animals, such as pets or wildlife, to identify their species or breeds. This is my study project for learning Web3, ICP and DeAI (Decentralized AI)

This project using two types of inference on-chain in blockchain 

1. Fully onchain inference using models trained off-chain: This assumes that the model was trained off-chain and then uploaded onchain. Inference happens fully onchain and has the same security and trustworthiness guarantees as regular smart contracts. ICP currently supports this use case for models with millions of parameters.

2. Smart contracts calling Web2 AI services: Smart contracts running on ICP can make HTTP requests to Web2 services including OpenAI and Claude. See an example of a smart contract calling the OpenAI API.

#### Stack
- mobilenetv2-7 (vision -- classifier)
- GPT-3.5-turbo (llm)
- ICP host
- Candid (Rust)
- Vanilla JS / HTML / CSS

#### Experience or lesson learned:
- I don't like it really much (web3 ecosystem), the documentation little bit unclear and outdated example. Finding almost 10 unique error from code, framework and package versioning, so many limitation coming from WebAssembly/web3.

#### **Key Features**:  
1. **Image Upload**:
   - Users can upload photos of animals for identification.
   
2. **Detailed Information**:
   - Provide the species/breed name.
   - Fun facts about the animal, habitat details, or breed-specific care tips.  

3. **Use Cases**:
   - Pet enthusiasts identifying breeds.
   - Wildlife explorers identifying animals during hikes or safaris.  

# Installation

Set system variable OPENAI_API_KEY since ICP doesn't support env::var()

```bash
export OPENAI_API_KEY=your_api_key
```

Install `dfx`, Rust, etc: https://internetcomputer.org/docs/current/developer-docs/getting-started/hello-world

Install WASI SDK 21:

- Install `wasi-skd-21.0` from https://github.com/WebAssembly/wasi-sdk/releases/tag/wasi-sdk-21
- Export `CC_wasm32_wasi` in your shell such that it points to WASI clang and sysroot. Example:

```
export CC_wasm32_wasi="/path/to/wasi-sdk-21.0/bin/clang --sysroot=/path/to/wasi-sdk-21.0/share/wasi-sysroot"
``` 

Install `wasi2ic`:
- Follow the steps in https://github.com/wasm-forge/wasi2ic
- Make sure that `wasi2ic` binary is in your `$PATH`.

Download MobileNet v2-7 to `src/backend/assets/mobilenetv2-7.onnx`:

```
./downdload_model.sh
```

Install NodeJS dependencies for the frontend:

```
npm install
```

Install `wasm-opt`:

```
cargo install wasm-opt
```

# Build

```
dfx start --background
dfx deploy
```

If the deployment is successful, the it will show the `frontend` URL.
Open that URL in browser to interact with the smart contract.
