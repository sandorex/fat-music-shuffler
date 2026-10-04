use std::io::{Cursor, Write, Read};

// TODO i just picked first value that worked for FAT32, as there are a lot of files in single directory
const BYTES_PER_SECTOR: u16 = 512;
const SECTOR_COUNT: u32 = 131070;
const FS_SIZE: usize = SECTOR_COUNT as usize * BYTES_PER_SECTOR as usize;

const REPEAT_COUNT: usize = 4;

#[test]
fn test_hardlink() {
    let _ = env_logger::builder().is_test(true).try_init();

    let mut buf: Vec<u8> = Vec::with_capacity(FS_SIZE);
    let mut image = Cursor::new(&mut buf);

    fatfs::format_volume(
        &mut fatfs::StdIoWrapper::from(&mut image),
        fatfs::FormatVolumeOptions::new()
            .fat_type(fatfs::FatType::Fat32) // FAT32 is required cause there is a lot of files
            .bytes_per_sector(BYTES_PER_SECTOR)
            .total_sectors(SECTOR_COUNT)
    )
    .expect("format volume");

    let fs = fatfs::FileSystem::new(image, fatfs::FsOptions::new())
        .expect("open fs");

    let root_dir = fs.root_dir();

    let original_dir = root_dir
        .create_dir("original")
        .expect("create dir original");

    let link_dir = root_dir
        .create_dir("link")
        .expect("create dir link");

    // create fake files
    for (i, file_name) in FILE_NAMES.iter().enumerate() {
        let mut file = original_dir
            .create_file(file_name)
            .expect(&format!("create file {file_name:?}"));

        file
            .write_all(&format!("file{}", i + 1).into_bytes())
            .expect(&format!("write to file {file_name:?}"));
    }

    let mut links: Vec<(String, String)> = vec![];

    for repeat in 0..REPEAT_COUNT {
        for (i, file_name) in FILE_NAMES.iter().enumerate() {
            links.push((format!("{}.mp3", 1 + i + FILE_NAMES.len() * repeat), file_name.to_string()));
        }
    }

    link_dir
        .create_hardlinks(&links, &original_dir)
        .expect("created hardlinks");

    // ignore previous and current directory
    let link_count = link_dir.iter().count() - 2;

    // ensure the same amount of links are created
    assert_eq!(link_count, links.len());

    // make sure first repeat of links is readable
    let mut buf = String::with_capacity(32);
    for i in 0..FILE_NAMES.len() {
        let link = format!("{}.mp3", i + 1);

        // remove previous file contents
        buf.clear();

        link_dir
            .open_file(&link)
            .expect(&format!("open link {link:?}"))
            .read_to_string(&mut buf)
            .expect(&format!("read link {link:?}"));

        assert_eq!(buf, format!("file{}", i + 1))
    }
}

const FILE_NAMES: &[&str] = &[
    "Rowing with One Hand [ymTDl537T24].mp3",
    "Nathan Evans - Wellerman (Sea Shanty) [qP-7GNoDJ5c].mp3",
    "The Chainsmokers, bludnymph - Self Destruction Mode (Official Video) [DrKA7sKOhws].mp3",
    "Hey, I don't work here - (Official Music Video) [wrJ6_GAprFE].mp3",
    "Neoni - WONDERLAND (Official Lyric Video) [0bTVSSiAgZs].mp3",
    "Wind Rose - The Returning Race [OFFICIAL MUSIC VIDEO] [U_YIbilsAgU].mp3",
    "My Mother Told Me (feat. Eric Hollaway) [siO_BSra0RI].mp3",
    "Xanadu by Ummet Ozcan - Full Version [KHn-rPbv0Sw].mp3",
    "Benny Benassi, The Biz - Satisfaction (Just____us Remix) [nwwZ4lv1ETw].mp3",
    "DAEGHO - BLUDLUST [1q4YY4Bsx_c].mp3",
    "Little Big & Little Sis Nora - Hardstyle Fish (Official Video) [pNwBeKfGOzQ].mp3",
    "Word of Baatyr [l-TNyw6J5n8].mp3",
    "Cristina Vee- Peaches (Bowsette version) [MoBpdoONxh8].mp3",
    "Friday Night Big Screen [MLzbL5tUWqw].mp3",
    "One More - S3RL & Atef ft Hannah Fortune & lowstattic [fcllKsaXwDY].mp3",
    "They Don't Care About Us [r1BcFD1g938].mp3",
    "SAINT [hYfQY0cLEDI].mp3",
    "Sophie Powers - Obsessed (feat. Ashley Sienna) [Official Visualizer] [uYSn6Igh-bk].mp3",
    "Fifth Harmony - I'm In Love With a Monster (from Hotel Transylvania 2 - Official Video) [xlkFTxnizyk].mp3",
    "Little Sis Nora, S3RL - UFO [Audio] [h8gKJFLFOlk].mp3",
    "r.i.p. [6eLMXab6ufU].mp3",
    "Charmes ft. Da Professor - Ready (Official Music Video) [cGAU_Xa-Xl0].mp3",
    "EMELINE - STRUT (Official Video) [5IDTWfLNnJE].mp3",
    "Little Mix - Sweet Melody (Official Video) [r4P-WOOUPk4].mp3",
    "Pyre Chant [hwvq976mf2Q].mp3",
    "W.I.T.C.H. - Devon Cole  Rock Version by Rain Paris [2ZRHy7t6wLI].mp3",
    "Ashnikko - Cheerleader (Official Music Video) [_BkjDZshvK8].mp3",
    "BABYMETAL - - Headbangeeeeerrrrr!!!!!!! (OFFICIAL) [2IzR_ClTE8Y].mp3",
    "Vnner och frnder (2023 Remastered) [yr7uhTB9OwQ].mp3",
    "AJR - World's Smallest Violin (Official Video) [PEnJbjBuxnw].mp3",
    "BLACKPINK - ' (PLAYING WITH FIRE)' MV [9pdj4iJD08s].mp3",
    "Pick Your Poison - A Cami-Cat Cover (Original by @Khamydrian ) [Mb3RlaRdL80].mp3",
    "Delilah Bon - Maverick (Official Music Video) [kvDqq9cc6ng].mp3",
    "Red Flag [s5L-rwmF1F8].mp3",
    "BABYMETAL x @ElectricCallboy - RATATATA (OFFICIAL VIDEO) [EDnIEWyVIlE].mp3",
    "bludnymph - FEAST (Let's Eat Yuh Yuh Music Video) [PrVlFrSv5oA].mp3",
    "Janar [8rLjLJyruOw].mp3",
    "Ljslfur [tVgUKJzDMnI].mp3",
    "Mneskin - I WANNA BE YOUR SLAVE (Official Video) [yOb9Xaug35M].mp3",
    "Barbie Girl - Metal Cover by Halocene (Aqua) [JHhzvRZ2j6M].mp3",
    "Sabrina Carpenter - Looking at Me (Audio Only) [lu9Ylxc0IZM].mp3",
    "Say So [mEhYtnHQAUw].mp3",
    "Tom MacDonald - People So Stupid [I6FmwBPDT-w].mp3",
    "Christus Vincit - Clamavi De Profundis [XNreKK--lYk].mp3",
    "Home Free - Skull and Bones (Official Lyric Video) [ob9zIqygSSc].mp3",
    "Ava Max - Who's Laughing Now [Official Music Video] [89S-RbszwJE].mp3",
    "Doki Doki  - S3RL ft Kawaiiconic [o5aQdXsCO2w].mp3",
    "Return To Sender [ji_WQtyjTAA].mp3",
    "Sex Sells (Hard) [TGU_fJvtzic].mp3",
    "Nitra, Hammer & Pick (Prod. TAURAS) [-UfofASS5BE].mp3",
    "The Dragonborn Comes [66G14_xLULk].mp3",
    "[MV] Solar()() _ HONEY() [-8cUVl2SzvA].mp3",
    "Genre Police - S3RL feat Lexi [IxGtRLM9HYc].mp3",
    "LISA - 'MONEY' EXCLUSIVE PERFORMANCE VIDEO [dNCWe_6HAM8].mp3",
    "GLORYHAMMER - Keeper Of The Celestial Flame Of Abernethy (Official Video)  Napalm Records [-O5TxWQkfr0].mp3",
    "marionette [Wpi0ZolLCRs].mp3",
    "[HyunA&DAWN] 'PING PONG' MV [0aaeUI1ucfQ].mp3",
    "AleXa ()  Bomb Official MV [2OjCMR8DGLg].mp3",
    "Ummet Ozcan -  Kayra (Official Music Video) [n21YEsQy67E].mp3",
    "    (   ) Otava Yo - russian couplets while fighting [0JQ0xnJyb0A].mp3",
    "GRANDADDY - The Horne Section  The Horne Section TV Show [27a-sCu9qhc].mp3",
    "Run & Hide [jkr6VurEXzg].mp3",
    "Downed And Drowned [DUr9FnXkuzs].mp3",
    "girli -  Ruthless (Official Music Video) [ltEnFK9S-Ig].mp3",
    "Helltaker Original - What the Hell (Metal Version) by Lollia, OR3O, Sleeping Forest feat. Friends [Oi2igPRbJwU].mp3",
    "Hunnu Guren - Batzorig Vaanchig & Auli [vztRqe_CHC0].mp3",
    "Little Sis Nora - Party Trick (Burp Song)  [Official Music Video] [EHj3BunC7Ek].mp3",
    "Swingrowers - Educated feet (Official MV) #electroswing [dSeuxL7Yu8A].mp3",
    "Ugly [kfatiYuyQKE].mp3",
    "Ay Kherel - MORGUL (Prayer) - Music.of.Tuva.(with lyrics) [mDWlyRABM1I].mp3",
    "Queen of Kings [IGSj-Noeb-M].mp3",
    "Cache & Djavo - Djavo nosi bradu (Official video 2021) [oegUTxyGZEg].mp3",
    "Delilah Bon - Evil, Hate Filled Female Official Music Video [o8mkNUYu0vg].mp3",
    "The Good Place Song  Pobodys Nerfect  Whitney Avalon [tPp-U4QonnM].mp3",
    "Alfons X HOURS - Davy Jones (Lyric Video) [QQjxKmAdVes].mp3",
    "Vana  - Clandestine (Official Music Video) [eJDqfQF_Oqc].mp3",
    "Vicious [eM9nmJkKpuA].mp3",
    "Snow Wife - ALL NIGHT (Official Music Video) [t28oPpUhZJo].mp3",
    "A SONGUS AMONGUS (Animation)  The Definitive Among Us Song.  feat. Black Gryph0n [bXE6B6Usj6o].mp3",
    "bludnymph - DarkMagicSillySexy (origin story) [Official Lyric Video] [90hFDQDGU-I].mp3",
    "Danheim & Skarphins son - Vgsp [iYKW8otERPk].mp3",
    "Elevate - Little Sis Nora & S3RL [24DDPKS8rQQ].mp3",
    "Feed the Machine [BlnVP2_dIb4].mp3",
    "Igowallah [MPBgDaTxIgQ].mp3",
    "BLACKPINK - 'Ice Cream (with Selena Gomez)' MV [vRXZj0DzXIA].mp3",
    "Home Free - Skull and Bones (Ubisoft)  Bass Singers Cover [igwTMPgujLQ].mp3",
    "Lee Man - Ne dam na kuma - Radijski Festival 2006 - (Tv S) [zAky0zpYlpA].mp3",
    "The Horse and the Infant [bWIgy-Ls-SU].mp3",
    "KONTRUST - Just Propaganda (Official Video)  Napalm Records [mIegk9Ukx4k].mp3",
    "LL - u turn me on (but u give me depression) (OFFICIAL VIDEO) [GuxyUEF3e6c].mp3",
    "Thunder [b_bEigUA1kk].mp3",
    "DeathbyRomy - Crash (Official Audio) [df_rOuJkWsQ].mp3",
    "Die for Me! [0tqEW0UxFjk].mp3",
    "Nathan Evans - The Last Shanty (Official Video) [zw0FZs_J2IE].mp3",
    "The Correspondents - Fear & Delight (Official Video) [ABS-mlep5rY].mp3",
    "XANA - BETTER KIND OF BEST FRIEND (Official Music Video) [Rwp4dnAgrbk].mp3",
    "HyunA _ Im Not Cool        Special Clip  Performance  P NATION [uoZKR8vA1xE].mp3",
    "Jagwar Twin - Bad Feeling (Oompa Loompa) (Official Music Video) [aPxFKQfv-Co].mp3",
    "Ram Jam - Black Betty [I_2D8Eo15wE].mp3",
    "Elverhy [3C73hUFokV4].mp3",
    "Mortal Kombat - After parti [61iNfpngw6g].mp3",
    "NATURE() LIMBO! () MV (Performance Ver.) [UK7yzgVpnDA].mp3",
    "PiNKII - Demon (Official Music Video) [dKsDHh2Vkv4].mp3",
    "bbno$ - it boy (official music video) [QuvqzlxEO6g].mp3",
    "bludnymph - Personal Pornstar [Official Lyric Video] [vC1rx_2G018].mp3",
    "Doja Cat - Get Into It (Yuh) (Official Video) [9Ko-nEYJ1GE].mp3",
    "Santiana [laY4TcWEbSA].mp3",
    "Puer Natus - Clamavi De Profundis [CBdp32lo_cU].mp3",
    "Naked in My Cellar [Qsd4hcGlh2w].mp3",
    "Ava Max - Maybe Youre The Problem (Official Video) [ijS_orLb6VU].mp3",
    "Cannibal [e3v60CXHnTA].mp3",
    "Em Beihold - Numb Little Bug [1fwJ8H5wWCU].mp3",
    "Everywhere I Go [U8pxJIYybko].mp3",
    "FEUERSCHWANZ - Warriors Of The World United (feat Thomas Winkler, Saltatio Mortis & Melissa Bonny) [-rKOoM7S6mw].mp3",
    "Halocene - Just Won't Die (Official Music Video) [Vguo2fT1YjE].mp3",
    "Sloppy Seconds (Ick Pt. 2) [4AAcgYxnYVc].mp3",
    "The Wellerman (Gingertail Cover) [MNmLn6a-jqw].mp3",
    "Namri - (Galadriel's Lament) - Clamavi De Profundis [re5_lzlFS9M].mp3",
    "Serebro - Mi Mi Mi (official video) [Iv4x0TC-1Qg].mp3",
    "THIS ISN'T ABOUT DRUGS (I SWEAR) (feat. From First to Last) [2E9nFQYZ3Ak].mp3",
    "CYN RAP  ASSIMILATE  RUSTAGE ft. Keetheweeb [MURDER DRONES] [c5ib8qAXafg].mp3",
    "Show Off [5Xf5B5SNTQg].mp3",
    "Tarantella [zSEaTRUEfjw].mp3",
    "WONDERLAND [uw9AOu0BRco].mp3",
    "Anya Nami - Dirty Dream (lyric video) [BWO-qeWj-nA].mp3",
    "Jessi () - 'ZOOM' MV [6j928wBZ_Bo].mp3",
    "OTYKEN - LEGEND (Official Music Video) [tXLoP9iSU5Y].mp3",
    "Prometheus [d5CqN12eaQM].mp3",
    "Barbaras Rhabarberbar [SLTyoIsCTGw].mp3",
    "Falling In Reverse - Watch The World Burn [qMXESlny4-I].mp3",
    "NCT 127 'Chain' MV [28XC2KRE-DE].mp3",
    "What Red Hot Chili Peppers sound like to people who don't like Red Hot Chili Peppers [VE5JMEu5hZA].mp3",
    "LITTLE BIG - TACOS (Official Music Video) [nUwTnJ8yFXY].mp3",
    "MORTEN feat. Frida Sundemo - Beautiful Heartbeat (Deorro Remix)  Music Visualization [nn7LeY3S72E].mp3",
    "Warrior of the Mind [oB8lqgO9e24].mp3",
    "Ennaria - Monstarrr - OFFICIAL MUSIC VIDEO [cNB4ucAvJ8g].mp3",
    "18+ [7FPdTV6-TWI].mp3",
    "Barbie & Ken [j8A2IKANrB4].mp3",
    "Crazy Bitch [LTulvuXrS_E].mp3",
    "DARKSIDE [jcKVfhr6O24].mp3",
    "bludnymph - Lights Out (Gen V Trailer Song) (Official Music Video) [9P_iVqrUvyM].mp3",
    "Cache & avo feat. Kruevac Geto - Automehaniar (Official video 2022) [lLzsFm4jk0E].mp3",
    "Clockwork God [LNRNVnL9rXs].mp3",
    "DON'T TOUCH MY CLOGS  OFFICIAL MUSIC VIDEO [dXqtrHJAqVM].mp3",
    "WET DREAM [HQO1BM00_lo].mp3",
    "COBRAH - GOOD PUSS (Official Music Video) [iEL5yL4aThc].mp3",
    "fuck getting better, I want to get worse [kzZjgUUH8to].mp3",
    "Mneskin - GOSSIP ft. Tom Morello [XrsbfrFPATs].mp3",
    "Snow Wife - AMERICAN HORROR SHOW (Official Music Video) [FH3CyuSzz98].mp3",
    "Custer [3pdVtlBYtrc].mp3",
    "Dove Cameron - Boyfriend (Official Audio) [ZGHRfLAn2wI].mp3",
    "Raganu Nakts [tIyYeA5olzY].mp3",
    "Shotgun Willy - Bombs Away (Official Music Video) [F2GhFXjQujI].mp3",
    "Panic Attacks in Paradise [lauT3tJ-kqE].mp3",
    "Pink (Freak) [MRODYhGguOk].mp3",
    "Pink Panther [D4Rr8Q9c3Mk].mp3",
    "Bears & Wolves [FnMFIpErRvs].mp3",
    "Doja Cat - Need to Know (Official Video) [dI3xkL7qUAc].mp3",
    "You and Me and the Devil Makes Three [rNkL8cDhhb0].mp3",
    "UNHOLY by Sam Smith  Female Cover by Justine M. (OC Lyrics) [uLAWHaiSC3s].mp3",
    "SPARKLE SONG - Entertaining  HalaCG (Honkai Star Rail) [Official MV] #MultiverseVistas [v_872GMoe08].mp3",
    "AronChupa, Flamingoz - Coco Song [Official Music Video] [B3M2trxQVH4].mp3",
    "BLACKPINK - Typa Girl (Official Audio) [UhxW9Njqqu0].mp3",
    "FRICTIONLESS WIPE  OFFICIAL VIDEO [LWD_4HcOLOY].mp3",
    "Doja Cat - Tia Tamera (Official Video) ft. Rico Nasty [C_yI2959DYU].mp3",
    "Fighting With The Melody [4SCQVGQaDfs].mp3",
    "LITTLE BIG  GENERATION CANCELLATION (Official Music Video) [7Yy4RP4FMNk].mp3",
    "Randy Dandy Oh [JqE9dsQH4AE].mp3",
    "ROACHES [Explicit Lyrics] - LuLuYam Official Song and Visualizer [a0Tne7IMX44].mp3",
    "AKN - NEEDED ME. [Ultra Records] [NlcHXGYtPqo].mp3",
    "Doja Cat - Rules (Official Video) [UVadfCxNnoY].mp3",
    "Dwarven Forge [k1gXmeihZfM].mp3",
    "METALI!! (feat. Tom Morello) [s9bQkfr1qZo].mp3",
    "Anya Nami - Bread (Official Music Video) [J1DAmmROUX8].mp3",
    "Jawbreaker [ruXdA3LTYNg].mp3",
    "Little Mix - Confetti (Official Video) ft. Saweetie [15-vLIZ_2mU].mp3",
    "Jessi () - 'X (What Type of X)' MV [OEu1OWf8ezU].mp3",
    "My Mother Told Me (Gingertail Cover) Vikings  Assassin's Creed Valhalla [KRqIkTlGIOE].mp3",
    "toki ala  toki pona music video [Lp-yvBl_VM0].mp3",
    "Vana - Ragdoll (Official Lyric Video) [ZtO5G4tJmJU].mp3",
    "MAUS MAKI - BOG I BATINA [PROD BY DARKO] (OFFICIAL LYRICS VIDEO) [KWgNhuB7zrk].mp3",
    "bludnymph - SEX MACHINA [Official Lyric Video] [jr1tPhA-xt0].mp3",
    "GASOLINE [KXvCS2KBbPw].mp3",
    "Voices In My Head [o6IIrkfq-MU].mp3",
    "DESINGERICA X DJEXON - PUMPALICCA (OFFICIAL VIDEO) [qnKcNP5YEgI].mp3",
    "Five Finger Death Punch - IOU (Official Lyric Video) [Yh98oDIi75w].mp3",
    "Haunted House [KCeYsyYo2gc].mp3",
    "Mneskin - DON'T WANNA SLEEP (Lyric Video) [q7xpwbdHAKM].mp3",
    "CHINCHILLA - Little Girl Gone (Official Music Video) [gnPKYVkK_iA].mp3",
    "Little Sis Nora - Limousine [Official Music Video] [dyGPeLeAYh0].mp3",
    "Daddy Cop (From the TV Show The Rookie) [qFmrO47hY9Y].mp3",
    "odjavna numera serije Komije [rWb7LX2snbo].mp3",
    "Therapy [gkTszt5XeJk].mp3",
    "ZITTI E BUONI [16GpicX09gk].mp3",
    "OTYKEN - STORM (Official Music Video) [CqwrwwOzVcQ].mp3",
    "Paranoia [7yp7HC9CdmU].mp3",
    "QUEEN OF THE FREAKS [UwN595hVR7U].mp3",
    "Superbeast [QC0BBgOdspo].mp3",
    "Sympathy [Explicit Lyrics] - LuLuYam Official Song and Visualizer [N1bAc-BKrjg].mp3",
    "()((G)I-DLE) - 'TOMBOY' Official Music Video [Jh4QFaPmdss].mp3",
    "Neoni - MACHINE [yDeH6JkHRoY].mp3",
    "The Tale of the Lesbian Hunter  Vinny Marchi [mfoq4Rskr-Q].mp3",
    " sprengisandi - Icelandic Folk Song [NTKtmQQaJYE].mp3",
    "Ezir-Kara (Black Eagle) [EQDnqyBEgkg].mp3",
    "Night Club - Candy Coated Suicide (Official Video) [NTcehtnfPGg].mp3",
    "2NE1 -    (I AM THE BEST) MV [j7_lSP8Vc3o].mp3",
    "GALXARA - Go Fck Myself (GFM) [OYkrSsY_MY0].mp3",
    "Scene Queen - Pink Bubblegum (Official Video) [0qGFBB1TGC0].mp3",
    "My Mother Told Me (Trance Viking) [dRVKk6LgKyY].mp3",
    "Devil (feat. Bubi) [Eu4z_n-9Vpw].mp3",
    "GLOW [PYHVZZyks3A].mp3",
    "Divine CEO [Explicit Lyrics] - LuLuYam Official Song and Visualizer [Q82T_yDHyM4].mp3",
    "Hinn Mikli Dreki [9-O_7TogedA].mp3",
    "Hot Mess - Zoe Clark [ZLH4s-15wd4].mp3",
    "TOMMY CASH - ESPRESSO MACCHIATO (Eurovision 2025 Winner) [o6aJJ6Q5zhg].mp3",
    "bludnymph - Grim Reaper (Official Lyric Video) [lvT0AwNH9xY].mp3",
    "AViVA - WEIRDO [S1Sn0Zc8jTI].mp3",
    "FEUERSCHWANZ - Ultima Nocte (Official Video)  Napalm Records [TxM-Yx0JzYI].mp3",
    "LITTLE BIG - Sck My Dck 2020 (Official Music Video) [7tThYxp5kmk].mp3",
    "Yung Gravy, bbno$ (BABY GRAVY) - You Need Jesus (Official Music Video) [Kalxiq1yGZU].mp3",
    "Ashnikko - You Make Me Sick! (Official Music Video) [yP6b4BeL6Zk].mp3",
    "BEG! [De5d6QeEjiM].mp3",
    "AleXa () - 'Wonderland' Official Performance Video [87Yui1Ff1AI].mp3",
    "Iggy Azalea - Started (Official Music Video) [flPCk8Z5XS0].mp3",
    "bludnymph - ORAL HEX (Official Lyric Video) [uHMfbqD8A7g].mp3",
    "Do I make you nervous [yT7NxBSwwXI].mp3",
    "Finally See Me [v-i3MASz4UE].mp3",
    "The Ballad of Sara Berry [XFcAKriYIAE].mp3",
    "Tujamo & LOTTEN - One Million (Official Music Video) [tcBVyspJYYw].mp3",
    "Jack Harris - Careful What You Wish For (the doctor said to) - Lyric Video [ZGbswRm4Vv0].mp3",
    "Phonk Rock Version [M7HbS2Z6JKA].mp3",
    "Bara Bada Bastu [QOehBj0nCMU].mp3",
    "Cal Scruby - MONEY BUY DRUGS (Official Music Video) [mZ4Mv8qnpOM].mp3",
    "PVRIS, Tommy Genesis, Alice Longyu Gao - Burn The Witch (OFFICIAL MUSIC VIDEO) [aye2CFuHr94].mp3",
    "The Tale Of C Chulainn by Miracle Of Sound (IRISHCELTIC FOLK METAL)) [XqyEADY_20Y].mp3",
    "Ummet Ozcan X Otyken   Altay  (Official Music Video) [aXsLlOPwe48].mp3",
    "Elena feat. Glance - Mamma mia (He's italiano) Official Video [iLZR7tihHbk].mp3",
    "TX2 - Degrade Me (Official Music Video) [6r79AhrOPOE].mp3",
    "Wynter Gordon - Dirty Talk (Izzamuzzic Remix)  Music Visualization [0yHqa8bHnZ0].mp3",
    "Anitta feat. Becky G - Banana [Official Music Video] [0OmRrFD8zJk].mp3",
    "Delilah Bon Ft ALT BLK ERA - WITCH (Lyric Video  Visualiser ) [eQPpF_pMuIw].mp3",
    "Heaven Was Full (I'm Headed Straight to Hell) [bb9aQy3d5oo].mp3",
    "Saweetie & GALXARA - Sway With Me (from Birds of Prey The Album) [Official Music Video] [19pp2jNapL4].mp3",
    "Villain [fv5OF9xhz4g].mp3",
    "Eva Simons - Policeman  feat. Konshens ( prod. by Sidney Samson ) [QLRWXD96_yU].mp3",
    "Saweetie - Best Friend (feat. Doja Cat) [Official Music Video] [_xJUCsyMQes].mp3",
    "[nendest] narkootikumidest ei tea me (kll) midagi [LUNapW1BXwY].mp3",
    "The Kingdom - Original Dwarven Song - Clamavi De Profundis [NylOKfrYR78].mp3",
    "Ummet Ozcan - Xanadu (Mongolian Techno) [9uMtnH7cABg].mp3",
    "The Drug In Me Is You [_CNipqLpmpU].mp3",
    "Weird Al Yankovic - First World Problems (Official 4K Video) [bwvlbJ0h35A].mp3",
    "Crazy Woman [VAjXtTwNo3w].mp3",
    "Queen of Nothing [MYVeUBtl5Xk].mp3",
    "Tardigrade Inferno - Ringmaster Has to Die (OFFICIAL MUSIC VIDEO) [hh3kZP4kNsE].mp3",
    "Altai Throat Singing [UaBPF5XGhqw].mp3",
    "FAUN - Diese kalte Nacht (Official Video) [zr8d9sXioj4].mp3",
    "Gnome - Wenceslas (Official Video) [LzG-qji05Lc].mp3",
    "LISA - 'LALISA' MV [awkkyBH2zEo].mp3",
    "OTYKEN - MY WING REMAKE popular on Tik Tok (official MV) [2BDSPZTqiUo].mp3",
    "Doja Cat - Paint The Town Red (Official Video) [m4_9TFeMfJE].mp3",
    "Erzhl es meinem Mittelfinger [m-2id6J83tE].mp3",
    "Pretty and Depressed [XFyoMu9K6ng].mp3",
    "Back of the Macca's - S3RL x Slen-D [wsOJ_gzjkIE].mp3",
    "BOWSETTE in 23 Animation Styles!  HUGE Community Collab  The Chalkeaters' Mario Song Remake [l-FDFsHA20w].mp3",
    "Hammer And The Anvil [O72EcodLzhk].mp3",
    "Ariana Grande - 7 rings (Official Video) [QYh6mYIJG2Y].mp3",
    "Becky G - Zooted (Official Video) ft. French Montana, Farruko [D91liF_Ml-M].mp3",
    "bludnymph - BODY PARTS [Official Lyric Video] [ytbNGacTS9Q].mp3",
    "AOA - Ai Wo Choudai (Dance Version) [x3kx5OV1zXo].mp3",
    "Dubioza Kolektiv - Balkan Boys (Official Video) [EL_UewkeDNY].mp3",
    "Subwoolfer - Give That Wolf A Banana (Official Music Video) [sDvXhZtcp0w].mp3",
    "The Ultimate Deception [XTfBMLC_kIo].mp3",
    "bludnymph - Watch Me (Official Music Video) [9iraxrhT1iw].mp3",
    "Lilyisthatyou - Moderation (Official Music Video) [7mMejHhd0eY].mp3",
    "Savage Daughter [nLWIvQmTo-k].mp3",
    "will.i.am - Scream & Shout ft. Britney Spears [kYtGl1dX5qI].mp3",
    "Becky G - MALA SANTA (lbum Visual) [w2Ro8cgsmss].mp3",
    "Mas Que Nada [eMt5IQWFTFw].mp3",
    "ALESTORM - Seventh Rum of a Seventh Rum (Official Video)  Napalm Records [OYLU_v2tOMI].mp3",
    "Bizarre - bludnymph feat. 6arelyhuman (Music Video) [F3u-l-jpupE].mp3",
    "FREAK (Feat. Hatsune Miku) [Explicit Lyrics] - LuLuYam Official Song and Visualizer [tpi1RwQBuPg].mp3",
    "HARLEQUIN! [6titLxDID9g].mp3",
    "DJ   A Remix DJ __Remix 2021 [us3ofM3zqXw].mp3",
    "Ummet Ozcan - A Shaman's Flute (Meditation Music) [sujU-jpiwqE].mp3",
    "CL - SPICY (Official Video) [QMwJtMJLXE0].mp3",
    "WTF 2020! (Funny Christmas Song for an Awful Year) explicit [zCsVp6hRTsA].mp3",
    "()((G)I-DLE) - 'MY BAG' (Choreography Practice Video) [Si5pQHRRH5w].mp3",
    "Cache & Djavo - Konobar (Official video 2017) [XpP6pT9ZLSg].mp3",
    "Aqua - Doctor Jones [-1jPUB7gRyg].mp3",
    "bludnymph - End Of The World (Official Music Video) [PWd-Dqvfb9g].mp3",
    "Bring Out Your Dead [lD1UqPHCpts].mp3",
    "Tuvan Throat Singing [qx8hrhBZJ98].mp3",
    "TX2 - Burn (Official Music Video) [WKkBvFZy6gE].mp3",
    "ASTON - For The Girls & Mama Didn't Raise No [Official Performance Video] [ITjpew-F-r4].mp3",
    "BLACKPINK - ' (AS IF IT'S YOUR LAST)' MV [Amq-qlqbjYA].mp3",
    "Cache & Djavo - Sekundarne sirovine (official video 2023) [xT2BT5T1HIA].mp3",
    "Neoni - Feet Don't Fail Me Now (Official Music Video) [RKsZAVNRxRQ].mp3",
    "PRETTYBOY! [cQc6Fu2ZidA].mp3",
    "Rauur Loginn Brann [5uMqY3WlDDA].mp3",
    "Blame It On The Kids [MCiDs3uoGk0].mp3",
    "bludnymph - Popsicle (Official Lyric Video) [GmjWIz6jkO8].mp3",
    "Elle King - Ex's & Oh's (Official Video) [0uLI6BnVh6w].mp3",
    "Hot Light & Slow Motion - Dynamite  Music Visualization [OXX8wxMs3Zo].mp3",
    "Ummet Ozcan - Bifrost (Viking Techno) [4oRLHlb5d5c].mp3",
    "Amelia Watson - Pop on Rocks A Dr. Seuss Rap [FZqK8jfbNbc].mp3",
    "Asja [7zi7aRGSAnA].mp3",
    "BRAND NEW BITCH [qogLbqsM4cA].mp3",
    "INSANE (A Hazbin Hotel Song) - Black Gryph0n & Baasik [juJkNKodgdE].mp3",
    "Konye [GVMe3FMucu4].mp3",
    "My Mother Told Me [iq6ZyHezloE].mp3",
    "Sofia Carson - Love Is the Name (Official Video) [fuyVJYP7GJk].mp3",
];
