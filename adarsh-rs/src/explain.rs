//! Longer explanations for error codes: `adarsh samjhao E001`.

pub struct Explanation {
    pub code: &'static str,
    pub title_hi: &'static str,
    pub title_en: &'static str,
    pub detail: &'static str,
    pub wrong: &'static str,
    pub right: &'static str,
}

macro_rules! e {
    ($code:literal, $hi:literal, $en:literal, $detail:literal, $wrong:literal, $right:literal) => {
        Explanation { code: $code, title_hi: $hi, title_en: $en, detail: $detail, wrong: $wrong, right: $right }
    };
}

pub static EXPLANATIONS: &[Explanation] = &[
    e!("E001", "Naam nahi mila", "Name not found",
       "Aapne aisa naam use kiya jo abhi tak banaya nahi gaya. Ya to spelling galat hai, ya 'badlo' se banana bhool gaye, ya woh kisi aur block ke andar bana hai jo yahan dikhta nahi.",
       "dikhao(umr);", "badlo umr = 24;\ndikhao(umr);"),
    e!("E002", "Galat number of arguments", "Wrong number of arguments",
       "Function ko jitne arguments chahiye, utne nahi diye (kam ya zyada). Function ki definition dekho: har parameter ko value chahiye, siwaay unke jinka default hai.",
       "kaam jodo(a, b) { wapas a + b; }\njodo(1);", "kaam jodo(a, b = 0) { wapas a + b; }\njodo(1);"),
    e!("E003", "Pakka value badli", "Changed a constant",
       "'pakka' se bani value kabhi nahi badalti. Agar badalni hai to 'badlo' use karo.",
       "pakka PI = 3.14;\nPI = 3;", "badlo pi = 3.14;\npi = 3;"),
    e!("E004", "Type nahi mila", "Unknown type",
       "Type ka naam (jaise number, string, list ya aapka dhacha) galat likha hai.",
       "badlo x: numbr = 5;", "badlo x: number = 5;"),
    e!("E005", "Loop ke bahar bas / aage_badho", "break/continue outside a loop",
       "'bas' loop ko rokta hai aur 'aage_badho' agle round par jaata hai, isliye yeh sirf jabtak, ginnati, ke_liye ya karo ke andar chalte hain. Function se bahar nikalne ke liye 'wapas' use karo.",
       "agar (x > 5) { bas; }", "jabtak (x > 5) { x -= 1; agar (x == 7) { bas; } }"),
    e!("E006", "Type match nahi hua", "Type mismatch",
       "Variable ya parameter par type likha hai (jaise ': number'), par value kisi aur type ki aayi.",
       "badlo umar: number = \"chaubees\";", "badlo umar: number = 24;"),
    e!("E007", "Module nahi mila", "Module not found",
       "'lao' ko file nahi mili. File ka naam aur jagah dekho: path us file ke folder ke hisaab se hota hai jo 'lao' likh rahi hai.",
       "lao \"utilz\";", "lao \"utils\";   # utils.aak isi folder mein ho"),
    e!("E008", "Dict mein key nahi", "Missing dictionary key",
       "Dict mein woh key hai hi nahi. Spelling dekho, ya pehle 'key in d' se jaancho, ya nikalo(d, key, default) use karo.",
       "badlo d = {\"naam\": \"A\"};\ndikhao(d[\"umar\"]);", "dikhao(nikalo(d, \"umar\", 0));"),
    e!("E009", "Function nahi hai", "Not callable",
       "Aapne kisi aisi cheez ke baad () lagaya jo function nahi hai, jaise number ya string.",
       "badlo x = 5;\nx();", "kaam x() { wapas 5; }\nx();"),
    e!("E010", "Recursion bahut gehri", "Recursion too deep",
       "Function khud ko itni baar call kar raha hai ki rukta hi nahi. Recursion mein ek 'base case' chahiye jahan woh wapas aa jaaye bina khud ko call kiye.",
       "kaam f(n) { wapas f(n - 1); }", "kaam f(n) { agar (n == 0) { wapas 0; } wapas f(n - 1); }"),
    e!("E011", "Index bahar", "Index out of range",
       "List ya string mein utne items nahi hain. Pehla index 0 hai, aakhri length - 1 (ya -1).",
       "badlo xs = [1, 2, 3];\ndikhao(xs[3]);", "dikhao(xs[2]);   # ya xs[-1]"),
    e!("E012", "Jaanch fail", "Assertion failed",
       "jaancho(sharat, sandesh) mein sharat jhuth nikli. Test ne bataya ki code ka result umeed se alag hai.",
       "jaancho(jodo(2, 2) == 5, \"jod galat\");", "jaancho(jodo(2, 2) == 4, \"jod galat\");"),
    e!("E100", "Syntax galat", "Syntax error",
       "Code ka dhaancha galat hai: koi bracket, ';' ya keyword gayab hai ya galat jagah hai. Caret (^) wali jagah aur uske theek pehle dekho.",
       "agar x > 5 { }", "agar (x > 5) { }"),
    e!("E101", "';' missing", "Missing semicolon",
       "Har statement ';' par khatam hota hai.",
       "badlo x = 5\ndikhao(x);", "badlo x = 5;\ndikhao(x);"),
    e!("E102", "Value ki jagah kuch aur", "Expected a value",
       "Yahan koi value ya expression (number, string, variable, function call) chahiye tha.",
       "badlo x = ;", "badlo x = 0;"),
    e!("W001", "Dhyan do (warning)", "Warning",
       "Yeh galti nahi hai, program chalega. Par yahan shayad kuch bhool hui hai: koi variable kabhi use nahi hua, kuch code kabhi nahi chalega, ya kisi builtin ka naam chhup gaya.",
       "badlo x = 5;   # x kabhi use nahi hua", "badlo _x = 5;   # '_' se shuru naam par warning nahi aati"),
];

pub fn find(code: &str) -> Option<&'static Explanation> {
    let code = code.trim().to_uppercase();
    EXPLANATIONS.iter().find(|e| e.code == code)
}

pub fn render(e: &Explanation) -> String {
    format!(
        "{}: {} ({})\n\n{}\n\nGalat:\n{}\n\nSahi:\n{}\n",
        e.code,
        e.title_hi,
        e.title_en,
        e.detail,
        indent(e.wrong),
        indent(e.right)
    )
}

fn indent(s: &str) -> String {
    s.lines().map(|l| format!("    {l}")).collect::<Vec<_>>().join("\n")
}
