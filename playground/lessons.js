// AdarshLang "Seekho" lessons for the web playground.
// Plain JS, one global: LESSONS.
var LESSONS = [
  {
    id: "pehla-program",
    title: "Pehla program (dikhao)",
    body: `<p><code>dikhao(...)</code> ka matlab hai screen par kuch likhna. Jo bhi tum likhoge, woh output mein dikhega.</p>
<p>Text ko double quotes <code>" "</code> mein likho, aur har statement ke end mein <code>;</code> lagao.</p>
<p>Har <code>dikhao</code> ek alag line print karta hai.</p>`,
    starter: `dikhao("Namaste Duniya");
# TODO: ek aur line dikhao: Aaj ka din shubh ho`,
    solution: `dikhao("Namaste Duniya");
dikhao("Aaj ka din shubh ho");`,
    check: { output: "Namaste Duniya\nAaj ka din shubh ho\n" },
    hint: 'Ek aur line ke liye dikhao("Aaj ka din shubh ho"); likho.'
  },
  {
    id: "variables",
    title: "Variables (badlo, pakka)",
    body: `<p><code>badlo</code> se variable banao. Ek variable ek box ki tarah hai jisme value rakhi jaati hai, aur value badal sakti hai.</p>
<p><code>pakka</code> wala variable constant hota hai: uski value kabhi nahin badalti. Badalne ki koshish karoge to error aayega.</p>
<p>String ke andar <code>{naam}</code> likho to variable ki value apne aap aa jaati hai.</p>`,
    starter: `badlo naam = "Riya";
pakka SHAHAR = "Delhi";
badlo umar = 0;  # TODO: umar ko 20 karo
dikhao("Mera naam {naam}, umar {umar}");`,
    solution: `badlo naam = "Riya";
pakka SHAHAR = "Delhi";
badlo umar = 20;
dikhao("Mera naam {naam}, umar {umar}");`,
    check: { output: "Mera naam Riya, umar 20\n" },
    hint: "umar = 0 ko 20 mein badlo."
  },
  {
    id: "numbers",
    title: "Numbers aur maths",
    body: `<p>Numbers par <code>+ - * /</code> chalte hain. <code>/</code> hamesha decimal answer deta hai: <code>17 / 4</code> ka answer <code>4.25</code> hai.</p>
<p><code>//</code> poora hissa deta hai (floor division): <code>17 // 5</code> ka answer <code>3</code> hai.</p>
<p><code>%</code> bachi hui value (remainder) deta hai: <code>17 % 5</code> ka answer <code>2</code> hai.</p>
<p><code>**</code> power hai: <code>2 ** 10</code> ka answer <code>1024</code> hai.</p>`,
    starter: `dikhao(17 / 5);   # TODO: yahan // use karo (poora hissa)
dikhao(17 % 5);
dikhao(2 * 10);   # TODO: yahan ** use karo (power)`,
    solution: `dikhao(17 // 5);
dikhao(17 % 5);
dikhao(2 ** 10);`,
    check: { output: "3\n2\n1024\n" },
    hint: "/ ki jagah // aur * ki jagah ** likho."
  },
  {
    id: "strings",
    title: "Strings aur interpolation",
    body: `<p>String ke andar <code>{variable}</code> likhoge to uski value wahan aa jaayegi. Ise <b>interpolation</b> kehte hain.</p>
<p>Number ka format bhi likh sakte ho: <code>{daam:.2}</code> ka matlab 2 decimal places, jaise <code>49.50</code>.</p>
<p><code>upper()</code> text ko bade letters mein badalta hai, <code>length()</code> letters ginta hai.</p>`,
    starter: `badlo naam = "adarsh";
badlo daam = 49.5;
badlo bada = naam;   # TODO: naam.upper() se bade letters karo
dikhao("Naam: {bada}");
dikhao("Daam: {daam}");   # TODO: {daam:.2} likho
# TODO: length dikhane ki ek line bhi likho`,
    solution: `badlo naam = "adarsh";
badlo daam = 49.5;
badlo bada = naam.upper();
dikhao("Naam: {bada}");
dikhao("Daam: {daam:.2}");
dikhao("Length: {naam.length()}");`,
    check: { output: "Naam: ADARSH\nDaam: 49.50\nLength: 6\n" },
    hint: "naam.upper() se bade letters aur {daam:.2} se 2 decimal."
  },
  {
    id: "agar-warna",
    title: "agar / warna",
    body: `<p><code>agar</code> ke andar ek condition hoti hai. Agar woh sach hai to pehla block chalega, warna <code>warna</code> wala block.</p>
<p>Chhota version bhi hai: <code>agar (umar >= 18) "adult" warna "bachcha"</code> ek value deta hai.</p>
<p>Comparison ke liye <code>==</code> (barabar), <code>!=</code> (barabar nahin), <code>&lt;</code>, <code>&gt;=</code> wagairah use karo.</p>`,
    starter: `badlo umar = 15;  # TODO: umar ko 20 karo
agar (umar >= 18) { dikhao("Vote de sakte ho"); } warna { dikhao("Abhi chhote ho"); }
dikhao(agar (umar >= 18) "adult" warna "bachcha");`,
    solution: `badlo umar = 20;
agar (umar >= 18) { dikhao("Vote de sakte ho"); } warna { dikhao("Abhi chhote ho"); }
dikhao(agar (umar >= 18) "adult" warna "bachcha");`,
    check: { output: "Vote de sakte ho\nadult\n" },
    hint: "umar = 15 ko 20 karo."
  },
  {
    id: "loops",
    title: "Loops: jabtak aur ginnati",
    body: `<p><code>jabtak (shart) { ... }</code> tab tak chalta hai jab tak shart sach hai.</p>
<p>Har baar loop chalne par counter badhana mat bhoolna, warna loop kabhi khatam nahin hoga!</p>
<p><code>ginnati (badlo i = 0; i &lt; 5; i += 1) { ... }</code> bhi ek loop hai, jo counter ke saath chalta hai.</p>`,
    starter: `badlo i = 1;
jabtak (i < 5) {   # TODO: 5 tak ginti chahiye, isliye condition ko <= 5 karo
    dikhao(i);
    i += 1;
}`,
    solution: `badlo i = 1;
jabtak (i <= 5) {
    dikhao(i);
    i += 1;
}`,
    check: { output: "1\n2\n3\n4\n5\n" },
    hint: "< ko <= kar do."
  },
  {
    id: "ke-liye-range",
    title: "ke_liye aur ranges",
    body: `<p><code>ke_liye (badlo x in list)</code> list ke har item par ek-ek karke chalta hai.</p>
<p><code>1..5</code> ek range hai: 1, 2, 3, 4, 5 (dono end shaamil). <code>5..1</code> ulta ginta hai.</p>
<p>Index bhi chahiye to do variables likho: <code>ke_liye (badlo i, phal in list)</code>. Pehla index <code>0</code> se shuru hota hai.</p>`,
    starter: `badlo phal = ["aam", "kela", "santra"];
ke_liye (badlo p in phal) {   # TODO: index bhi chahiye: ke_liye (badlo i, p in phal)
    dikhao(p);
}`,
    solution: `badlo phal = ["aam", "kela", "santra"];
ke_liye (badlo i, p in phal) {
    dikhao("{i}: {p}");
}`,
    check: { output: "0: aam\n1: kela\n2: santra\n" },
    hint: "Loop mein do variables likho: i aur p."
  },
  {
    id: "lists",
    title: "Lists (push, sort, sum)",
    body: `<p>List <code>[ ]</code> mein kai values ek saath rakhte hain. Pehla item index <code>0</code> par hota hai.</p>
<p><code>push(list, value)</code> end mein naya item jodta hai. <code>list[-1]</code> aakhri item deta hai.</p>
<p><code>sort(list)</code> items ko chhote se bade karta hai, <code>sum(list)</code> sab ka jod deta hai, aur <code>list[0:2]</code> pehle do items (slice) deta hai.</p>`,
    starter: `badlo nums = [5, 2, 9];
# TODO: push(nums, 1) se 1 list mein jodo
dikhao(sort(nums));
dikhao(sum(nums));`,
    solution: `badlo nums = [5, 2, 9];
push(nums, 1);
dikhao(sort(nums));
dikhao(sum(nums));`,
    check: { output: "[1, 2, 5, 9]\n17\n" },
    hint: "push(nums, 1); sort se pehle likho."
  },
  {
    id: "dicts",
    title: "Dicts (keys, values)",
    body: `<p>Dict <code>{"key": value}</code> mein naam aur value ki jodi hoti hai.</p>
<p>Naya key jodne ke liye <code>d["shahar"] = "Indore"</code> likho. Value padhne ke liye <code>d["naam"]</code> ya seedha <code>d.naam</code>.</p>
<p><code>ke_liye (badlo k, v in d)</code> har key aur uski value par chalta hai.</p>`,
    starter: `badlo d = {"naam": "Adarsh", "umar": 24};
# TODO: d mein "shahar": "Indore" jodo
ke_liye (badlo k, v in d) {
    dikhao("{k}: {v}");
}`,
    solution: `badlo d = {"naam": "Adarsh", "umar": 24};
d["shahar"] = "Indore";
ke_liye (badlo k, v in d) {
    dikhao("{k}: {v}");
}`,
    check: { output: "naam: Adarsh\numar: 24\nshahar: Indore\n" },
    hint: 'd["shahar"] = "Indore"; likho.'
  },
  {
    id: "functions",
    title: "Functions (kaam, wapas)",
    body: `<p><code>kaam</code> se function banate hain. <code>wapas</code> se answer wapas milta hai.</p>
<p>Default value: <code>naam = "dost"</code> likho to argument na dene par yahi value use hoti hai.</p>
<p>Named arguments se order ki chinta nahin: <code>jodo(b: 1, a: 2)</code> bhi chalega.</p>`,
    starter: `# TODO: kaam salaam(naam) banao jo "Namaste {naam}!" wapas kare
kaam salaam(naam) {
    wapas "Namaste {naam}!";
}
dikhao(salaam("Riya"));
dikhao(salaam());   # TODO: naam ki default value "dost" do`,
    solution: `kaam salaam(naam = "dost") {
    wapas "Namaste {naam}!";
}
dikhao(salaam("Riya"));
dikhao(salaam());`,
    check: { output: "Namaste Riya!\nNamaste dost!\n" },
    hint: 'kaam salaam(naam = "dost") likho.'
  },
  {
    id: "lambdas",
    title: "Lambdas aur map, filter, reduce",
    body: `<p>Lambda ek chhota function hai: <code>x => x * 2</code>.</p>
<p><code>map</code> har item ko badalta hai, <code>filter</code> sirf kaam ke items rakhta hai, aur <code>reduce</code> sab ko ek value mein jodta hai.</p>
<p>Dot se bhi likh sakte ho: <code>nums.map(x => x * 2)</code>.</p>`,
    starter: `badlo nums = [1, 2, 3, 4, 5];
dikhao(nums.map(x => x * 1));   # TODO: har number ko 2 se guna karo
dikhao(nums.filter(x => x > 10));   # TODO: sirf even numbers (x % 2 == 0) rakho`,
    solution: `badlo nums = [1, 2, 3, 4, 5];
dikhao(nums.map(x => x * 2));
dikhao(nums.filter(x => x % 2 == 0));`,
    check: { output: "[2, 4, 6, 8, 10]\n[2, 4]\n" },
    hint: "map mein x * 2 aur filter mein x % 2 == 0 likho."
  },
  {
    id: "dhacha",
    title: "Dhacha (records aur methods)",
    body: `<p><code>dhacha</code> ek naya type banata hai, jaise <code>Vyakti</code> jisme <code>naam</code> aur <code>umar</code> hon.</p>
<p>Method <code>kaam Vyakti.parichay()</code> ke andar <code>yeh</code> us record ko kehta hai, jisse uske fields padh sakte ho.</p>
<p>Record banane ke liye: <code>Vyakti { naam: "Riya", umar: 20 }</code>.</p>`,
    starter: `dhacha Vyakti { naam, umar = 0 };

kaam Vyakti.parichay() {
    # TODO: "Main {yeh.naam}, umar {yeh.umar}" wapas karo
    wapas "";
}

badlo v = Vyakti { naam: "Riya", umar: 20 };
dikhao(v.parichay());`,
    solution: `dhacha Vyakti { naam, umar = 0 };

kaam Vyakti.parichay() {
    wapas "Main {yeh.naam}, umar {yeh.umar}";
}

badlo v = Vyakti { naam: "Riya", umar: 20 };
dikhao(v.parichay());`,
    check: { output: "Main Riya, umar 20\n" },
    hint: 'wapas "Main {yeh.naam}, umar {yeh.umar}"; likho.'
  },
  {
    id: "errors",
    title: "Errors (pakdo, chhoddo)",
    body: `<p>Kabhi kabhi program mein galti hoti hai. <code>chhoddo("sandesh")</code> se tum khud ek galti bhejte ho.</p>
<p><code>pakdo { ... } chhoddo (e) { ... }</code> se galti pakad lete ho, taaki program achanak band na ho.</p>
<p><code>aakhir { ... }</code> hamesha chalta hai, galti ho ya na ho.</p>`,
    starter: `kaam umar_check(u) {
    agar (u < 0) {
        chhoddo("Umar negative nahin ho sakti");
    }
    wapas u;
}

# TODO: neeche wali line ko pakdo { } mein lapeto, chhoddo (e) bhi likho
dikhao(umar_check(-5));`,
    solution: `kaam umar_check(u) {
    agar (u < 0) {
        chhoddo("Umar negative nahin ho sakti");
    }
    wapas u;
}

pakdo {
    dikhao(umar_check(-5));
} chhoddo (e) {
    dikhao("Galti: {e}");
} aakhir {
    dikhao("Check khatam");
}`,
    check: { output: "Galti: Umar negative nahin ho sakti\nCheck khatam\n" },
    hint: "pakdo { ... } chhoddo (e) { dikhao(...) } aakhir { ... } likho."
  },
  {
    id: "kachhua",
    title: "Kachhua (turtle drawing)",
    body: `<p><code>kachhua</code> ek chhota turtle hai jo pen se drawing karta hai.</p>
<p><code>aage(100)</code> aage jaata hai, aur <code>daayen(90)</code> daayen (right) ghoomta hai. Square banane ke liye 4 baar aage aur 90 daayen karo.</p>
<p>Loop se yeh aasaan ho jaata hai: <code>ginnati (badlo i = 0; i &lt; 4; i += 1)</code>. Program ke end mein <code>dikhao("ban gaya")</code> likho.</p>`,
    starter: `kachhua.rang("laal");
kachhua.aage(100);
# TODO: baaki 3 side banao (aage aur daayen), phir aakhir mein dikhao("ban gaya");`,
    solution: `kachhua.rang("laal");
ginnati (badlo i = 0; i < 4; i += 1) {
    kachhua.aage(100);
    kachhua.daayen(90);
}
dikhao("ban gaya");`,
    check: { contains: ["ban gaya"] },
    hint: "4 baar aage(100) aur daayen(90) karo, phir dikhao(\"ban gaya\")."
  }
];
