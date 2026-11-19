// AdarshLang playground gallery: example programs, one global (GALLERY).
// Each code string is a complete program that runs without keyboard input.
var GALLERY = [
  {
    id: "namaste",
    title: "Namaste, Priya",
    category: "Shuruaat",
    description: "Variables, string interpolation and a simple agar/warna check.",
    code: `badlo naam = "Priya";
badlo shahar = "Jaipur";
badlo umar = 19;

dikhao("Namaste {naam}!");
dikhao("Aap {shahar} se ho? Bahut accha!");
dikhao("Umar: {umar} saal");
dikhao("Naam ki lambai: {naam.length()} akshar");
dikhao(naam.upper() + " ... ji haan!");

agar (umar >= 18) {
    dikhao("{naam}, aap vote de sakti ho.");
} warna {
    dikhao("{naam}, abhi thoda intezaar karo.");
}

dikhao("{"Chai"} ke bina {naam} ka din shuru nahin hota.");
`
  },
  {
    id: "chai-bill",
    title: "Chai Corner ka Bill",
    category: "Shuruaat",
    description: "A chai shop bill with rupee formatting and 5 percent GST.",
    code: `badlo chai = 15;
badlo samosa = 20;
badlo bun_maska = 35;

badlo chai_qty = 3;
badlo samosa_qty = 2;
badlo bun_qty = 1;

badlo subtotal = chai * chai_qty + samosa * samosa_qty + bun_maska * bun_qty;
badlo gst = subtotal * 0.05;
badlo kul = subtotal + gst;

dikhao("==== Gupta Chai Corner ====");
dikhao("Chai      x{chai_qty}  {rupaye(chai * chai_qty)}");
dikhao("Samosa    x{samosa_qty}  {rupaye(samosa * samosa_qty)}");
dikhao("Bun maska x{bun_qty}  {rupaye(bun_maska * bun_qty)}");
dikhao("------------------------");
dikhao("Subtotal: {rupaye(subtotal)}");
dikhao("GST 5%:   {rupaye(gst)}");
dikhao("Kul:      {rupaye(kul)}");
dikhao("Dhanyavaad, phir aana!");
`
  },
  {
    id: "pahada",
    title: "Pahada: Sattar ka",
    category: "Shuruaat",
    description: "A times table using loops and format specs for padding.",
    code: `badlo n = 7;

dikhao("Pahada {n} ka");
dikhao("-------------");

ke_liye (badlo i in 1..10) {
    badlo phal = i * n;
    dikhao("{i:02} x {n} = {phal:>3}");
}

badlo bada = 10 * n * n;
dikhao("Dus baar {n} ka {n} guna: {bada:,}");
badlo vishm = filter(x => x % 2 == 1, 1..10);
dikhao("Vishm ank (odd): {vishm}");
`
  },
  {
    id: "guess-bot",
    title: "Chatur Bot",
    category: "Games aur Fun",
    description: "A bot guesses a secret number from 1 to 100 using binary search.",
    code: `badlo raaz = random_int(1, 100);
badlo lo = 1;
badlo hi = 100;
badlo koshish = 0;
badlo mila = jhuth;

dikhao("Maine ek number socha hai (1 se 100 ke beech). Bot dhoondhega!");

jabtak (!mila && lo <= hi) {
    badlo andaaza = (lo + hi) // 2;
    koshish += 1;
    agar (andaaza == raaz) {
        mila = sahi_hai_be;
        dikhao("Koshish {koshish}: bot ne {andaaza} bola ... sahi!");
    } warna agar (andaaza < raaz) {
        dikhao("Koshish {koshish}: {andaaza} chhota hai, ab bada dhoondho");
        lo = andaaza + 1;
    } warna {
        dikhao("Koshish {koshish}: {andaaza} bada hai, ab chhota dhoondho");
        hi = andaaza - 1;
    }
}

dikhao("Bot ne {raaz} ko {koshish} koshish mein dhoondh liya.");
`
  },
  {
    id: "patthar-kagaz-kainchi",
    title: "Patthar Kagaz Kainchi",
    category: "Games aur Fun",
    description: "Five rounds of rock-paper-scissors with a vikalp enum and chuno.",
    code: `vikalp Chaal { Patthar, Kagaz, Kainchi };

# Kaun si chaal is chaal ko haraati hai?
kaam haraane_wali(chaal) {
    chuno (chaal) {
        case Chaal.Patthar: wapas Chaal.Kagaz;
        case Chaal.Kagaz: wapas Chaal.Kainchi;
        warna_case: wapas Chaal.Patthar;
    }
}

kaam natija(a, b) {
    agar (a == b) { wapas "barabar"; }
    agar (haraane_wali(a) == b) { wapas "Sonu jeeta"; }
    wapas "Raju jeeta";
}

badlo chaalein = [Chaal.Patthar, Chaal.Kagaz, Chaal.Kainchi];
badlo raju_jeet = 0;
badlo sonu_jeet = 0;

ginnati (badlo round = 1; round <= 5; round += 1) {
    badlo raju = random_choice(chaalein);
    badlo sonu = random_choice(chaalein);
    badlo phaisla = natija(raju, sonu);
    dikhao("Round {round}: Raju={raju.naam}, Sonu={sonu.naam} -> {phaisla}");
    agar (phaisla == "Raju jeeta") { raju_jeet += 1; }
    agar (phaisla == "Sonu jeeta") { sonu_jeet += 1; }
}

dikhao("Score: Raju {raju_jeet}, Sonu {sonu_jeet}");
`
  },
  {
    id: "fizzbuzz-chai",
    title: "FizzBuzz: Chai Samosa",
    category: "Games aur Fun",
    description: "FizzBuzz with Chai for threes, Samosa for fives and both for fifteens.",
    code: `kaam fizz_buzz(n) {
    agar (n % 15 == 0) { wapas "Chai Samosa"; }
    agar (n % 3 == 0) { wapas "Chai"; }
    agar (n % 5 == 0) { wapas "Samosa"; }
    wapas "{n}";
}

dikhao("FizzBuzz, chai-samosa wala:");
badlo chai_count = 0;
badlo samosa_count = 0;

ke_liye (badlo i in 1..20) {
    badlo shabd = fizz_buzz(i);
    dikhao("{i:>2}: {shabd}");
    agar (shabd == "Chai") { chai_count += 1; }
    agar (shabd == "Samosa") { samosa_count += 1; }
}

dikhao("Chai aaya {chai_count} baar, Samosa {samosa_count} baar.");
`
  },
  {
    id: "primes",
    title: "Primes ki Chhalni",
    category: "Maths",
    description: "Filters primes with is_prime and finds twin prime pairs.",
    code: `badlo primes = filter(is_prime, 1..60);
dikhao("1 se 60 tak prime: {primes}");
dikhao("Kul prime: {length(primes)}");

badlo jodiyan = [];
ke_liye (badlo p in primes) {
    agar (p + 2 in primes) {
        push(jodiyan, p);
    }
}
dikhao("Twin prime shuruaat (p, p+2): {jodiyan}");
dikhao("Sabse bada prime: {primes[-1]}");

badlo sum_prime = sum(primes);
dikhao("Sabhi primes ka jod: {sum_prime}");
`
  },
  {
    id: "fibonacci",
    title: "Fibonacci Generator",
    category: "Maths",
    description: "A generator function built with dete_raho, then filtered and summed.",
    code: `kaam fibonacci(n) {
    badlo a = 0;
    badlo b = 1;
    ke_liye (badlo i in 1..n) {
        dete_raho a;
        badlo agla = a + b;
        a = b;
        b = agla;
    }
}

dikhao("Pehle 12 Fibonacci ank: {fibonacci(12)}");

badlo sab = fibonacci(30);
dikhao("Kul ank: {length(sab)}, aakhri: {sab[-1]}");

badlo juft = filter(x => x % 2 == 0, sab);
dikhao("Juft fibonacci: {juft}");
dikhao("Golden ratio ka paas: {sab[-1] / sab[-2]:.6}");
dikhao("Yog: {sum(sab):,}");
`
  },
  {
    id: "shabd-lakh",
    title: "Bade Ank aur Shabd",
    category: "Maths",
    description: "Indian number words, lakh grouping, gcd, lcm and 30 factorial.",
    code: `dikhao("gcd(48, 18) = {gcd(48, 18)}");
dikhao("lcm(4, 6) = {lcm(4, 6)}");

badlo ank = [1, 21, 101, 1250];
ke_liye (badlo n in ank) {
    dikhao("{n} = {shabdon_mein(n)}");
}

dikhao("Bharat mein: {lakh_format(1500000)} rupaye");
dikhao("Ek crore: {lakh_format(10000000)}");

badlo factorial = reduce((a, b) => a * b, 1..30, 1);
dikhao("30! = {factorial}");
dikhao("30! mein {length(shabdme(factorial))} ank hain");
`
  },
  {
    id: "marks-report",
    title: "Class ka Marks Report",
    category: "Data",
    description: "Sorts student records by total marks and prints an aligned report.",
    code: `badlo bachche = [
    {"naam": "Riya", "maths": 92, "hindi": 78},
    {"naam": "Arjun", "maths": 65, "hindi": 88},
    {"naam": "Meera", "maths": 81, "hindi": 94},
    {"naam": "Vikram", "maths": 47, "hindi": 59}
];

badlo kramit = sort(bachche, by: b => b.maths + b.hindi, ulta: sahi_hai_be);

dikhao("{"Naam":<8} | {"Maths":>5} | {"Hindi":>5} | {"Pratishat":>9}");
dikhao("---------+-------+-------+----------");
ke_liye (badlo b in kramit) {
    badlo pratishat = (b.maths + b.hindi) / 200;
    dikhao("{b.naam:<8} | {b.maths:>5} | {b.hindi:>5} | {pratishat:>9.1%}");
}

badlo topper = kramit[0];
dikhao("Topper: {topper.naam}");
`
  },
  {
    id: "json-chai",
    title: "JSON ka Menu",
    category: "Data",
    description: "Writes a shop menu as JSON, reads it back and totals the prices.",
    code: `badlo menu = {
    "shop": "Gupta Chai Corner",
    "khulta": "subah 6 baje",
    "items": ["chai", "samosa", "bun maska"],
    "daam": {"chai": 15, "samosa": 20, "bun maska": 35}
};

badlo text = json_banao(menu, sundar: sahi_hai_be);
dikhao(text);

badlo padha = json_padho(text);
dikhao("Shop: {padha.shop}");
dikhao("Pehla item: {padha.items[0]}");

badlo total = 0;
ke_liye (badlo naam, daam in padha.daam) {
    total += daam;
    dikhao("{naam} -> {rupaye(daam)}");
}
dikhao("Total daam: {rupaye(total)}");
`
  },
  {
    id: "diwali-countdown",
    title: "Diwali ka Countdown",
    category: "Data",
    description: "Counts days to Diwali from a fixed date using tareekh and din_ka_fark.",
    code: `badlo aaj = tareekh(2026, 10, 1);
badlo diwali = tareekh(2026, 11, 8);

kaam dikhao_tareekh(t) { wapas "{t.din}/{t.mahina}/{t.saal}"; }

dikhao("Aaj ki tareekh: {dikhao_tareekh(aaj)}, din: {din_ka_naam(aaj)}");
dikhao("Diwali: {dikhao_tareekh(diwali)}, din: {din_ka_naam(diwali)}");

badlo bache = din_ka_fark(aaj, diwali);
dikhao("Aaj se Diwali tak {bache} din baaki hain!");

badlo dhanteras = tareekh_jodo(diwali, -2);
dikhao("Dhanteras: {dikhao_tareekh(dhanteras)} ({din_ka_naam(dhanteras)})");

badlo ghar_saaf = tareekh_jodo(aaj, 14);
dikhao("Do hafte mein ghar saaf karna: {dikhao_tareekh(ghar_saaf)}");
`
  },
  {
    id: "kachhua-taara",
    title: "Kachhue ka Taara",
    category: "Drawing (kachhua)",
    description: "The turtle draws a five-point star in yellow.",
    code: `kachhua.rang("peela");
kachhua.mota(2);

ginnati (badlo i = 0; i < 5; i += 1) {
    kachhua.aage(120);
    kachhua.daayen(144);
}

kachhua.kalam_upar();
kachhua.ghar();
kachhua.kalam_neeche();
dikhao("Taara ban gaya! 5 kone, 5 lakeeren.");
`
  },
  {
    id: "kachhua-spiral",
    title: "Kachhue ki Spiral",
    category: "Drawing (kachhua)",
    description: "A square spiral that grows and changes colour as it goes.",
    code: `kachhua.mota(1);
badlo lambai = 5;
badlo rang_list = ["laal", "narangi", "peela", "hara", "neela"];

ginnati (badlo i = 0; i < 20; i += 1) {
    kachhua.rang(rang_list[i % 5]);
    kachhua.aage(lambai);
    kachhua.daayen(90);
    lambai += 6;
}

dikhao("Spiral poori hui: {20} kadam, aakhri lambai {lambai - 6}");
`
  },
  {
    id: "kachhua-rangoli",
    title: "Rangoli ka Phool",
    category: "Drawing (kachhua)",
    description: "Twelve overlapping circles that make a flower, each in a rangoli colour.",
    code: `badlo rang_list = ["laal", "peela", "hara", "neela", "gulabi", "narangi"];
kachhua.mota(1);

ginnati (badlo phool = 0; phool < 12; phool += 1) {
    kachhua.rang(rang_list[phool % 6]);
    ginnati (badlo kadam = 0; kadam < 36; kadam += 1) {
        kachhua.aage(2);
        kachhua.daayen(10);
    }
    kachhua.daayen(30);
}

dikhao("Rangoli taiyaar: {12} phool, {6} rang");
`
  },
  {
    id: "paisa-records",
    title: "Paisa ka Hisaab",
    category: "Advanced",
    description: "A record with operator methods, custom text and inheritance.",
    code: `dhacha Paisa { rupaye };
dhacha Bachat : Paisa { mahine = 1 };

kaam Paisa.jodo(other) { wapas Paisa { rupaye: yeh.rupaye + other.rupaye }; }
kaam Paisa.ghatao(other) { wapas Paisa { rupaye: yeh.rupaye - other.rupaye }; }
kaam Paisa.shabd() { wapas "{rupaye(yeh.rupaye)}"; }

badlo chai = Paisa { rupaye: 120 };
badlo samosa = Paisa { rupaye: 80 };

dikhao("Chai + samosa = {chai + samosa}");
dikhao("Chai - samosa = {chai - samosa}");
dikhao("Chai ka paisa: {chai}");
dikhao("Barabar hai? {chai == Paisa { rupaye: 120 }}");

badlo jama = Bachat { rupaye: 500, mahine: 3 };
dikhao("Bachat record hai? {jama hai Paisa}");
dikhao("{jama.mahine} mahine ka {jama.rupaye} rupaye");
`
  },
  {
    id: "bank-khata",
    title: "Bank ka Khata",
    category: "Advanced",
    description: "Typed errors with dhacha, chhoddo and pakdo catch clauses.",
    code: `dhacha BalanceKam { kami };

kaam nikaalo(balance, rakam) {
    agar (rakam > balance) {
        chhoddo(BalanceKam { kami: rakam - balance });
    }
    wapas balance - rakam;
}

badlo khata = 500;
dikhao("Shuruaati balance: {rupaye(khata)}");

ke_liye (badlo rakam in [200, 250, 300]) {
    pakdo {
        khata = nikaalo(khata, rakam);
        dikhao("{rupaye(rakam)} nikale, bacha: {rupaye(khata)}");
    } chhoddo (e: BalanceKam) {
        dikhao("Balance kam hai! {rupaye(e.kami)} kam pade.");
    } aakhir {
        dikhao("Ek transaction poora hua.");
    }
}

dikhao("Aakhir balance: {rupaye(khata)}");
`
  },
  {
    id: "list-chuno",
    title: "Chuno: List ki Pehchaan",
    category: "Advanced",
    description: "Pattern matching on lists, ranges, types and guards with chuno.",
    code: `kaam pehchaano(cheez) {
    chuno (cheez) {
        case []: wapas "khali list";
        case [ek]: wapas "akela ek: {ek}";
        case [pehla, dusra]: wapas "do cheezein: {pehla} aur {dusra}";
        case [pehla, ...bache]: wapas "{pehla} se shuru, {length(bache)} aur";
        case 0: wapas "zero";
        case string: wapas "shabd: {cheez}";
        case 1..9: wapas "chhota ank";
        case x agar x < 0: wapas "negative ank";
        warna_case: wapas "kuch aur";
    }
}

dikhao(pehchaano([]));
dikhao(pehchaano([7]));
dikhao(pehchaano([1, 2]));
dikhao(pehchaano([5, 6, 7, 8]));
dikhao(pehchaano(0));
dikhao(pehchaano(4));
dikhao(pehchaano(-12));
dikhao(pehchaano("Adarsh"));
`
  }
];
