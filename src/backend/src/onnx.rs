use crate::Classification;
use prost::Message;
use std::cell::RefCell;
use tract_onnx::prelude::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use dotenv::dotenv;

type Model = SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

thread_local! {
    static MODEL: RefCell<Option<Model>> = RefCell::new(None);
}

#[derive(Serialize)]
struct CompletionRequest {
    model: String,
    messages: Vec<Messag>,
    max_tokens: u32,
}

#[derive(Serialize)]
struct Messag {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: MessageResponse,
}

#[derive(Deserialize)]
struct MessageResponse {
    content: String,
}


/// An image classification model trained on ImageNet dataset.
/// The model itself is not included in the source code of this example.
/// If you see a compile error here, then download the model from:
/// https://github.com/onnx/models/tree/main/validated/vision/classification/mobilenet
/// See the `download_model.sh` script for details.
const IMAGENET: &'static [u8] = include_bytes!("../assets/mobilenetv2-7.onnx");

/// Constructs a runnable model from the serialized ONNX model in `IMAGENET`.
pub fn setup() -> TractResult<()> {
    let bytes = bytes::Bytes::from_static(IMAGENET);
    let proto: tract_onnx::pb::ModelProto = tract_onnx::pb::ModelProto::decode(bytes)?;
    let model = tract_onnx::onnx()
        .model_for_proto_model(&proto)?
        .into_optimized()?
        .into_runnable()?;
    MODEL.with_borrow_mut(|m| {
        *m = Some(model);
    });
    Ok(())
}

/// Runs the model on the given image and returns top three labels.
pub fn classify(image: Vec<u8>) -> Result<Vec<Classification>, anyhow::Error> {
    MODEL.with_borrow(|model| {
        let model = model.as_ref().unwrap();
        let image = image::load_from_memory(&image)?.to_rgb8();

        // The model accepts an image of size 224x224px.
        let image =
            image::imageops::resize(&image, 224, 224, ::image::imageops::FilterType::Triangle);

        // Preprocess the input according to
        // https://github.com/onnx/models/tree/main/validated/vision/classification/mobilenet#preprocessing.
        const MEAN: [f32; 3] = [0.485, 0.456, 0.406];
        const STD: [f32; 3] = [0.229, 0.224, 0.225];
        let tensor = tract_ndarray::Array4::from_shape_fn((1, 3, 224, 224), |(_, c, y, x)| {
            (image[(x as u32, y as u32)][c] as f32 / 255.0 - MEAN[c]) / STD[c]
        });

        let result = model.run(tvec!(Tensor::from(tensor).into()))?;

        let mut scores: Vec<_> = result[0]
            .to_array_view::<f32>()?
            .into_iter()
            .zip(0..)
            .collect();

        scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        let labels = scores
            .iter()
            .take(1)
            .map(|(score, i)| Classification {
                label: LABELS[*i as usize].to_string(),
                score: **score,
            })
            .collect();
        Ok(labels)
    })
}

#[ic_cdk::query]
fn gateway_http_request() -> String {
    "hello".to_string()
}

#[ic_cdk::query]
pub async fn send_http_post_request(prompt: String) -> String {
    dotenv().ok();
    let api_key = env::var("OPENAI_API_KEY").expect("Cannot find OPENAI API KEY");
    let client = Client::new();

    let messages = vec![
        Messag {
            role: "system".to_string(),
            content: "I want you to act as a pet behaviorist. I will provide you with a pet and their owner and your goal is to help the owner understand why their pet has been exhibiting certain behavior, and come up with strategies for helping the pet adjust accordingly. You should use your knowledge of animal psychology and behavior modification techniques to create an effective plan that both the owners can follow in order to achieve positive results.".to_string(),
        },
        Messag {
            role: "user".to_string(),
            content: prompt
        }
    ];

    let request_body = CompletionRequest {
        model: "gpt-3.5-turbo".to_string(),
        messages,
        max_tokens: 150,
    };

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await;

    let chat_response: ChatResponse = response.expect("Error response").json().await.expect("Error parsing");

    if let Some(choice) = chat_response.choices.get(0) {
        choice.message.content.to_string()
    } else {
        "error".to_string()
    }
}

/// The set of 1000 ImageNet class labels.
const LABELS: [&'static str; 1000] = [
    "tench",
    "goldfish",
    "great white shark",
    "tiger shark",
    "hammerhead",
    "electric ray",
    "stingray",
    "cock",
    "hen",
    "ostrich",
    "brambling",
    "goldfinch",
    "house finch",
    "junco",
    "indigo bunting",
    "robin",
    "bulbul",
    "jay",
    "magpie",
    "chickadee",
    "water ouzel",
    "kite",
    "bald eagle",
    "vulture",
    "great grey owl",
    "european fire salamander",
    "common newt",
    "eft",
    "spotted salamander",
    "axolotl",
    "bullfrog",
    "tree frog",
    "tailed frog",
    "loggerhead",
    "leatherback turtle",
    "mud turtle",
    "terrapin",
    "box turtle",
    "banded gecko",
    "common iguana",
    "american chameleon",
    "whiptail",
    "agama",
    "frilled lizard",
    "alligator lizard",
    "gila monster",
    "green lizard",
    "african chameleon",
    "komodo dragon",
    "african crocodile",
    "american alligator",
    "triceratops",
    "thunder snake",
    "ringneck snake",
    "hognose snake",
    "green snake",
    "king snake",
    "garter snake",
    "water snake",
    "vine snake",
    "night snake",
    "boa constrictor",
    "rock python",
    "indian cobra",
    "green mamba",
    "sea snake",
    "horned viper",
    "diamondback",
    "sidewinder",
    "trilobite",
    "harvestman",
    "scorpion",
    "black and gold garden spider",
    "barn spider",
    "garden spider",
    "black widow",
    "tarantula",
    "wolf spider",
    "tick",
    "centipede",
    "black grouse",
    "ptarmigan",
    "ruffed grouse",
    "prairie chicken",
    "peacock",
    "quail",
    "partridge",
    "african grey",
    "macaw",
    "sulphur-crested cockatoo",
    "lorikeet",
    "coucal",
    "bee eater",
    "hornbill",
    "hummingbird",
    "jacamar",
    "toucan",
    "drake",
    "red-breasted merganser",
    "goose",
    "black swan",
    "tusker",
    "echidna",
    "platypus",
    "wallaby",
    "koala",
    "wombat",
    "jellyfish",
    "sea anemone",
    "brain coral",
    "flatworm",
    "nematode",
    "conch",
    "snail",
    "slug",
    "sea slug",
    "chiton",
    "chambered nautilus",
    "dungeness crab",
    "rock crab",
    "fiddler crab",
    "king crab",
    "american lobster",
    "spiny lobster",
    "crayfish",
    "hermit crab",
    "isopod",
    "white stork",
    "black stork",
    "spoonbill",
    "flamingo",
    "little blue heron",
    "american egret",
    "bittern",
    "crane",
    "limpkin",
    "european gallinule",
    "american coot",
    "bustard",
    "ruddy turnstone",
    "red-backed sandpiper",
    "redshank",
    "dowitcher",
    "oystercatcher",
    "pelican",
    "king penguin",
    "albatross",
    "grey whale",
    "killer whale",
    "dugong",
    "sea lion",
    "chihuahua",
    "japanese spaniel",
    "maltese dog",
    "pekinese",
    "shih-tzu",
    "blenheim spaniel",
    "papillon",
    "toy terrier",
    "rhodesian ridgeback",
    "afghan hound",
    "basset",
    "beagle",
    "bloodhound",
    "bluetick",
    "black-and-tan coonhound",
    "walker hound",
    "english foxhound",
    "redbone",
    "borzoi",
    "irish wolfhound",
    "italian greyhound",
    "whippet",
    "ibizan hound",
    "norwegian elkhound",
    "otterhound",
    "saluki",
    "scottish deerhound",
    "weimaraner",
    "staffordshire bullterrier",
    "american staffordshire terrier",
    "bedlington terrier",
    "border terrier",
    "kerry blue terrier",
    "irish terrier",
    "norfolk terrier",
    "norwich terrier",
    "yorkshire terrier",
    "wire-haired fox terrier",
    "lakeland terrier",
    "sealyham terrier",
    "airedale",
    "cairn",
    "australian terrier",
    "dandie dinmont",
    "boston bull",
    "miniature schnauzer",
    "giant schnauzer",
    "standard schnauzer",
    "scotch terrier",
    "tibetan terrier",
    "silky terrier",
    "soft-coated wheaten terrier",
    "west highland white terrier",
    "lhasa",
    "flat-coated retriever",
    "curly-coated retriever",
    "golden retriever",
    "labrador retriever",
    "chesapeake bay retriever",
    "german short-haired pointer",
    "vizsla",
    "english setter",
    "irish setter",
    "gordon setter",
    "brittany spaniel",
    "clumber",
    "english springer",
    "welsh springer spaniel",
    "cocker spaniel",
    "sussex spaniel",
    "irish water spaniel",
    "kuvasz",
    "schipperke",
    "groenendael",
    "malinois",
    "briard",
    "kelpie",
    "komondor",
    "old english sheepdog",
    "shetland sheepdog",
    "collie",
    "border collie",
    "bouvier des flandres",
    "rottweiler",
    "german shepherd",
    "doberman",
    "miniature pinscher",
    "greater swiss mountain dog",
    "bernese mountain dog",
    "appenzeller",
    "entlebucher",
    "boxer",
    "bull mastiff",
    "tibetan mastiff",
    "french bulldog",
    "great dane",
    "saint bernard",
    "eskimo dog",
    "malamute",
    "siberian husky",
    "dalmatian",
    "affenpinscher",
    "basenji",
    "pug",
    "leonberg",
    "newfoundland",
    "great pyrenees",
    "samoyed",
    "pomeranian",
    "chow",
    "keeshond",
    "brabancon griffon",
    "pembroke",
    "cardigan",
    "toy poodle",
    "miniature poodle",
    "standard poodle",
    "mexican hairless",
    "timber wolf",
    "white wolf",
    "red wolf",
    "coyote",
    "dingo",
    "dhole",
    "african hunting dog",
    "hyena",
    "red fox",
    "kit fox",
    "arctic fox",
    "grey fox",
    "tabby",
    "tiger cat",
    "persian cat",
    "siamese cat",
    "egyptian cat",
    "cougar",
    "lynx",
    "leopard",
    "snow leopard",
    "jaguar",
    "lion",
    "tiger",
    "cheetah",
    "brown bear",
    "american black bear",
    "ice bear",
    "sloth bear",
    "mongoose",
    "meerkat",
    "tiger beetle",
    "ladybug",
    "ground beetle",
    "long-horned beetle",
    "leaf beetle",
    "dung beetle",
    "rhinoceros beetle",
    "weevil",
    "fly",
    "bee",
    "ant",
    "grasshopper",
    "cricket",
    "walking stick",
    "cockroach",
    "mantis",
    "cicada",
    "leafhopper",
    "lacewing",
    "dragonfly",
    "damselfly",
    "admiral",
    "ringlet",
    "monarch",
    "cabbage butterfly",
    "sulphur butterfly",
    "lycaenid",
    "starfish",
    "sea urchin",
    "sea cucumber",
    "wood rabbit",
    "hare",
    "angora",
    "hamster",
    "porcupine",
    "fox squirrel",
    "marmot",
    "beaver",
    "guinea pig",
    "sorrel",
    "zebra",
    "hog",
    "wild boar",
    "warthog",
    "hippopotamus",
    "ox",
    "water buffalo",
    "bison",
    "ram",
    "bighorn",
    "ibex",
    "hartebeest",
    "impala",
    "gazelle",
    "arabian camel",
    "llama",
    "weasel",
    "mink",
    "polecat",
    "black-footed ferret",
    "otter",
    "skunk",
    "badger",
    "armadillo",
    "three-toed sloth",
    "orangutan",
    "gorilla",
    "chimpanzee",
    "gibbon",
    "siamang",
    "guenon",
    "patas",
    "baboon",
    "macaque",
    "langur",
    "colobus",
    "proboscis monkey",
    "marmoset",
    "capuchin",
    "howler monkey",
    "titi",
    "spider monkey",
    "squirrel monkey",
    "madagascar cat",
    "indri",
    "indian elephant",
    "african elephant",
    "lesser panda",
    "giant panda",
    "barracouta",
    "eel",
    "coho",
    "rock beauty",
    "anemone fish",
    "sturgeon",
    "gar",
    "lionfish",
    "puffer",
    "abacus",
    "abaya",
    "academic gown",
    "accordion",
    "acoustic guitar",
    "aircraft carrier",
    "airliner",
    "airship",
    "altar",
    "ambulance",
    "amphibian",
    "analog clock",
    "apiary",
    "apron",
    "ashcan",
    "assault rifle",
    "backpack",
    "bakery",
    "balance beam",
    "balloon",
    "ballpoint",
    "band aid",
    "banjo",
    "bannister",
    "barbell",
    "barber chair",
    "barbershop",
    "barn",
    "barometer",
    "barrel",
    "barrow",
    "baseball",
    "basketball",
    "bassinet",
    "bassoon",
    "bathing cap",
    "bath towel",
    "bathtub",
    "beach wagon",
    "beacon",
    "beaker",
    "bearskin",
    "beer bottle",
    "beer glass",
    "bell cote",
    "bib",
    "bicycle-built-for-two",
    "bikini",
    "binder",
    "binoculars",
    "birdhouse",
    "boathouse",
    "bobsled",
    "bolo tie",
    "bonnet",
    "bookcase",
    "bookshop",
    "bottlecap",
    "bow",
    "bow tie",
    "brass",
    "brassiere",
    "breakwater",
    "breastplate",
    "broom",
    "bucket",
    "buckle",
    "bulletproof vest",
    "bullet train",
    "butcher shop",
    "cab",
    "caldron",
    "candle",
    "cannon",
    "canoe",
    "can opener",
    "cardigan",
    "car mirror",
    "carousel",
    "carpenter's kit",
    "carton",
    "car wheel",
    "cash machine",
    "cassette",
    "cassette player",
    "castle",
    "catamaran",
    "cd player",
    "cello",
    "cellular telephone",
    "chain",
    "chainlink fence",
    "chain mail",
    "chain saw",
    "chest",
    "chiffonier",
    "chime",
    "china cabinet",
    "christmas stocking",
    "church",
    "cinema",
    "cleaver",
    "cliff dwelling",
    "cloak",
    "clog",
    "cocktail shaker",
    "coffee mug",
    "coffeepot",
    "coil",
    "combination lock",
    "computer keyboard",
    "confectionery",
    "container ship",
    "convertible",
    "corkscrew",
    "cornet",
    "cowboy boot",
    "cowboy hat",
    "cradle",
    "crane",
    "crash helmet",
    "crate",
    "crib",
    "crock pot",
    "croquet ball",
    "crutch",
    "cuirass",
    "dam",
    "desk",
    "desktop computer",
    "dial telephone",
    "diaper",
    "digital clock",
    "digital watch",
    "dining table",
    "dishrag",
    "dishwasher",
    "disk brake",
    "dock",
    "dogsled",
    "dome",
    "doormat",
    "drilling platform",
    "drum",
    "drumstick",
    "dumbbell",
    "dutch oven",
    "electric fan",
    "electric guitar",
    "electric locomotive",
    "entertainment center",
    "envelope",
    "espresso maker",
    "face powder",
    "feather boa",
    "file",
    "fireboat",
    "fire engine",
    "fire screen",
    "flagpole",
    "flute",
    "folding chair",
    "football helmet",
    "forklift",
    "fountain",
    "fountain pen",
    "four-poster",
    "freight car",
    "french horn",
    "frying pan",
    "fur coat",
    "garbage truck",
    "gasmask",
    "gas pump",
    "goblet",
    "go-kart",
    "golf ball",
    "golfcart",
    "gondola",
    "gong",
    "gown",
    "grand piano",
    "greenhouse",
    "grille",
    "grocery store",
    "guillotine",
    "hair slide",
    "hair spray",
    "half track",
    "hammer",
    "hamper",
    "hand blower",
    "hand-held computer",
    "handkerchief",
    "hard disc",
    "harmonica",
    "harp",
    "harvester",
    "hatchet",
    "holster",
    "home theater",
    "honeycomb",
    "hook",
    "hoopskirt",
    "horizontal bar",
    "horse cart",
    "hourglass",
    "ipod",
    "iron",
    "jack-o'-lantern",
    "jean",
    "jeep",
    "jersey",
    "jigsaw puzzle",
    "jinrikisha",
    "joystick",
    "kimono",
    "knee pad",
    "knot",
    "lab coat",
    "ladle",
    "lampshade",
    "laptop",
    "lawn mower",
    "lens cap",
    "letter opener",
    "library",
    "lifeboat",
    "lighter",
    "limousine",
    "liner",
    "lipstick",
    "loafer",
    "lotion",
    "loudspeaker",
    "loupe",
    "lumbermill",
    "magnetic compass",
    "mailbag",
    "mailbox",
    "maillot",
    "maillot",
    "manhole cover",
    "maraca",
    "marimba",
    "mask",
    "matchstick",
    "maypole",
    "maze",
    "measuring cup",
    "medicine chest",
    "megalith",
    "microphone",
    "microwave",
    "military uniform",
    "milk can",
    "minibus",
    "miniskirt",
    "minivan",
    "missile",
    "mitten",
    "mixing bowl",
    "mobile home",
    "model t",
    "modem",
    "monastery",
    "monitor",
    "moped",
    "mortar",
    "mortarboard",
    "mosque",
    "mosquito net",
    "motor scooter",
    "mountain bike",
    "mountain tent",
    "mouse",
    "mousetrap",
    "moving van",
    "muzzle",
    "nail",
    "neck brace",
    "necklace",
    "nipple",
    "notebook",
    "obelisk",
    "oboe",
    "ocarina",
    "odometer",
    "oil filter",
    "organ",
    "oscilloscope",
    "overskirt",
    "oxcart",
    "oxygen mask",
    "packet",
    "paddle",
    "paddlewheel",
    "padlock",
    "paintbrush",
    "pajama",
    "palace",
    "panpipe",
    "paper towel",
    "parachute",
    "parallel bars",
    "park bench",
    "parking meter",
    "passenger car",
    "patio",
    "pay-phone",
    "pedestal",
    "pencil box",
    "pencil sharpener",
    "perfume",
    "petri dish",
    "photocopier",
    "pick",
    "pickelhaube",
    "picket fence",
    "pickup",
    "pier",
    "piggy bank",
    "pill bottle",
    "pillow",
    "ping-pong ball",
    "pinwheel",
    "pirate",
    "pitcher",
    "plane",
    "planetarium",
    "plastic bag",
    "plate rack",
    "plow",
    "plunger",
    "polaroid camera",
    "pole",
    "police van",
    "poncho",
    "pool table",
    "pop bottle",
    "pot",
    "potter's wheel",
    "power drill",
    "prayer rug",
    "printer",
    "prison",
    "projectile",
    "projector",
    "puck",
    "punching bag",
    "purse",
    "quill",
    "quilt",
    "racer",
    "racket",
    "radiator",
    "radio",
    "radio telescope",
    "rain barrel",
    "recreational vehicle",
    "reel",
    "reflex camera",
    "refrigerator",
    "remote control",
    "restaurant",
    "revolver",
    "rifle",
    "rocking chair",
    "rotisserie",
    "rubber eraser",
    "rugby ball",
    "rule",
    "running shoe",
    "safe",
    "safety pin",
    "saltshaker",
    "sandal",
    "sarong",
    "sax",
    "scabbard",
    "scale",
    "school bus",
    "schooner",
    "scoreboard",
    "screen",
    "screw",
    "screwdriver",
    "seat belt",
    "sewing machine",
    "shield",
    "shoe shop",
    "shoji",
    "shopping basket",
    "shopping cart",
    "shovel",
    "shower cap",
    "shower curtain",
    "ski",
    "ski mask",
    "sleeping bag",
    "slide rule",
    "sliding door",
    "slot",
    "snorkel",
    "snowmobile",
    "snowplow",
    "soap dispenser",
    "soccer ball",
    "sock",
    "solar dish",
    "sombrero",
    "soup bowl",
    "space bar",
    "space heater",
    "space shuttle",
    "spatula",
    "speedboat",
    "spider web",
    "spindle",
    "sports car",
    "spotlight",
    "stage",
    "steam locomotive",
    "steel arch bridge",
    "steel drum",
    "stethoscope",
    "stole",
    "stone wall",
    "stopwatch",
    "stove",
    "strainer",
    "streetcar",
    "stretcher",
    "studio couch",
    "stupa",
    "submarine",
    "suit",
    "sundial",
    "sunglass",
    "sunglasses",
    "sunscreen",
    "suspension bridge",
    "swab",
    "sweatshirt",
    "swimming trunks",
    "swing",
    "switch",
    "syringe",
    "table lamp",
    "tank",
    "tape player",
    "teapot",
    "teddy",
    "television",
    "tennis ball",
    "thatch",
    "theater curtain",
    "thimble",
    "thresher",
    "throne",
    "tile roof",
    "toaster",
    "tobacco shop",
    "toilet seat",
    "torch",
    "totem pole",
    "tow truck",
    "toyshop",
    "tractor",
    "trailer truck",
    "tray",
    "trench coat",
    "tricycle",
    "trimaran",
    "tripod",
    "triumphal arch",
    "trolleybus",
    "trombone",
    "tub",
    "turnstile",
    "typewriter keyboard",
    "umbrella",
    "unicycle",
    "upright",
    "vacuum",
    "vase",
    "vault",
    "velvet",
    "vending machine",
    "vestment",
    "viaduct",
    "violin",
    "volleyball",
    "waffle iron",
    "wall clock",
    "wallet",
    "wardrobe",
    "warplane",
    "washbasin",
    "washer",
    "water bottle",
    "water jug",
    "water tower",
    "whiskey jug",
    "whistle",
    "wig",
    "window screen",
    "window shade",
    "windsor tie",
    "wine bottle",
    "wing",
    "wok",
    "wooden spoon",
    "wool",
    "worm fence",
    "wreck",
    "yawl",
    "yurt",
    "web site",
    "comic book",
    "crossword puzzle",
    "street sign",
    "traffic light",
    "book jacket",
    "menu",
    "plate",
    "guacamole",
    "consomme",
    "hot pot",
    "trifle",
    "ice cream",
    "ice lolly",
    "french loaf",
    "bagel",
    "pretzel",
    "cheeseburger",
    "hotdog",
    "mashed potato",
    "head cabbage",
    "broccoli",
    "cauliflower",
    "zucchini",
    "spaghetti squash",
    "acorn squash",
    "butternut squash",
    "cucumber",
    "artichoke",
    "bell pepper",
    "cardoon",
    "mushroom",
    "granny smith",
    "strawberry",
    "orange",
    "lemon",
    "fig",
    "pineapple",
    "banana",
    "jackfruit",
    "custard apple",
    "pomegranate",
    "hay",
    "carbonara",
    "chocolate sauce",
    "dough",
    "meat loaf",
    "pizza",
    "potpie",
    "burrito",
    "red wine",
    "espresso",
    "cup",
    "eggnog",
    "alp",
    "bubble",
    "cliff",
    "coral reef",
    "geyser",
    "lakeside",
    "promontory",
    "sandbar",
    "seashore",
    "valley",
    "volcano",
    "ballplayer",
    "groom",
    "scuba diver",
    "rapeseed",
    "daisy",
    "yellow lady's slipper",
    "corn",
    "acorn",
    "hip",
    "buckeye",
    "coral fungus",
    "agaric",
    "gyromitra",
    "stinkhorn",
    "earthstar",
    "hen-of-the-woods",
    "bolete",
    "ear",
    "toilet tissue",
];
//1. IMPORT IC MANAGEMENT CANISTER
//This includes all methods and types needed
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
    TransformContext,
};

use ic_cdk_macros::{self, query};
use serde_json::{self};

// This struct is legacy code and is not really used in the code.
#[derive(Serialize, Deserialize)]
struct Context {
    bucket_start_time_index: usize,
    closing_price_index: usize,
}

//Update method using the HTTPS outcalls feature
#[ic_cdk::update]
async fn llm(prompt: String) -> String {
    //2. SETUP ARGUMENTS FOR HTTP GET request

    // 2.1 Setup the URL
    let api_key = env!("OPENAI_API_KEY").to_string();
    let url = "https://api.openai.com/v1/chat/completions";
    // 2.2 prepare headers for the system http_request call
    //Note that `HttpHeader` is declared in line 4
    let request_headers = vec![
        HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
        HttpHeader {
            name: "Authorization".to_string(),
            value: format!("Bearer {}", api_key),
        },
    ];

    //note "CanisterHttpRequestArgument" and "HttpMethod" are declared in line 4.
    //CanisterHttpRequestArgument has the following types:

    // pub struct CanisterHttpRequestArgument {
    //     pub url: String,
    //     pub max_response_bytes: Option<u64>,
    //     pub method: HttpMethod,
    //     pub headers: Vec<HttpHeader>,
    //     pub body: Option<Vec<u8>>,
    //     pub transform: Option<TransformContext>,
    // }
    //see: https://docs.rs/ic-cdk/latest/ic_cdk/api/management_canister/http_request/struct.CanisterHttpRequestArgument.html

    //Where "HttpMethod" has structure:
    // pub enum HttpMethod {
    //     GET,
    //     POST,
    //     HEAD,
    // }
    //See: https://docs.rs/ic-cdk/latest/ic_cdk/api/management_canister/http_request/enum.HttpMethod.html

    //Since the body in HTTP request has type Option<Vec<u8>> it needs to look something like this: Some(vec![104, 101, 108, 108, 111]) ("hello" in ASCII)
    //where the vector of u8s are the UTF. In order to send JSON via POST we do the following:
    //1. Declare a JSON string to send
    //2. Convert that JSON string to array of UTF8 (u8)
    //3. Wrap that array in an optional
    let json_string : String = format!("{{ \"model\": \"gpt-3.5-turbo\", \"messages\": [ {{ \"role\": \"system\", \"content\": \"I want you to act as a pet behaviorist. I will provide you with a pet and your goal is to help the owner understand why their pet has been exhibiting certain behavior, and come up with strategies for helping the pet adjust accordingly. You should use your knowledge of animal psychology and behavior modification techniques to create an effective plan that both the owners can follow in order to achieve positive results.\" }}, {{ \"role\": \"user\", \"content\": \"{}\" }} ] }}", prompt);

    //note: here, r#""# is used for raw strings in Rust, which allows you to include characters like " and \ without needing to escape them.
    //We could have used "serde_json" as well.
    let json_utf8: Vec<u8> = json_string.into_bytes();
    let request_body: Option<Vec<u8>> = Some(json_utf8);

    // This struct is legacy code and is not really used in the code. Need to be removed in the future
    // The "TransformContext" function does need a CONTEXT parameter, but this implementation is not necessary
    // the TransformContext(transform, context) below accepts this "context", but it does nothing with it in this implementation.
    // bucket_start_time_index and closing_price_index are meaninglesss
    let context = Context {
        bucket_start_time_index: 0,
        closing_price_index: 4,
    };

    let request = CanisterHttpRequestArgument {
        url: url.to_string(),
        max_response_bytes: None, //optional for request
        method: HttpMethod::POST,
        headers: request_headers,
        body: request_body,
        transform: Some(TransformContext::new(transform, serde_json::to_vec(&context).unwrap())),
        // transform: None, //optional for request
    };

    //3. MAKE HTTPS REQUEST AND WAIT FOR RESPONSE

    //Note: in Rust, `http_request()` already sends the cycles needed
    //so no need for explicit Cycles.add() as in Motoko
    match http_request(request).await {
        //4. DECODE AND RETURN THE RESPONSE

        //See:https://docs.rs/ic-cdk/latest/ic_cdk/api/management_canister/http_request/struct.HttpResponse.html
        Ok((response,)) => {
            //if successful, `HttpResponse` has this structure:
            // pub struct HttpResponse {
            //     pub status: Nat,
            //     pub headers: Vec<HttpHeader>,
            //     pub body: Vec<u8>,
            // }

            //We need to decode that Vec<u8> that is the body into readable text.
            //To do this, we:
            //  1. Call `String::from_utf8()` on response.body
            //  3. We use a switch to explicitly call out both cases of decoding the Blob into ?Text
            let str_body = String::from_utf8(response.body)
                .expect("Transformed response is not UTF-8 encoded.");
            ic_cdk::api::print(format!("{:?}", str_body));

            //The API response will looks like this:
            // { successful: true }

            //Return the body as a string and end the method
            let result: String = format!(
                "{}. See more info of the request sent at: {}/inspect",
                str_body, url
            );
            result
        }
        Err((r, m)) => {
            let message =
                format!("The http_request resulted into error. RejectionCode: {r:?}, Error: {m}");

            //Return the error as a string and end the method
            message
        }
    }

}

// Strips all data that is not needed from the original response.
#[query]
fn transform(raw: TransformArgs) -> HttpResponse {

    let headers = vec![];
    

    let mut res = HttpResponse {
        status: raw.response.status.clone(),
        body: raw.response.body.clone(),
        headers,
        ..Default::default()
    };

    if res.status == 200 {

        res.body = raw.response.body;
    } else {
        ic_cdk::api::print(format!("Received an error from coinbase: err = {:?}", raw));
    }
    res
}
