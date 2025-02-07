

pub const EMOJIS: phf::Map<&'static str, &'static str> = phf::phf_map! {
    "white_check_mark" => "\u{2705}", // ✅
    "eyes" => "\u{1f440}", // 👀
    "+1" => "\u{1f44d}", // 👍
    "-1" => "\u{1f44e}", // 👎
    "sparkles" => "\u{2728}", // ✨
    "wink" => "\u{1f609}", // 😉
    "heart" => "\u{2764}", // ❤
    "fire" => "\u{1f525}", // 🔥
    "confused" => "\u{1f615}", // 😕
    "sob" => "\u{1f62d}", // 😭
    "clap" => "\u{1f44f}", // 👏
    "grinning" => "\u{1f600}", // 😀
    "sweat_smile" => "\u{1f605}", // 😅
    "tada" => "\u{1f389}", // 🎉
    "laughing" => "\u{1f606}", // 😆
    "heart_eyes" => "\u{1f60d}", // 😍
    "kissing_heart" => "\u{1f618}", // 😘
    "kissing" => "\u{1f617}", // 😗
    "kissing_smiling_eyes" => "\u{1f619}", // 😙
    "kissing_closed_eyes" => "\u{1f61a}", // 😚
    "stuck_out_tongue_winking_eye" => "\u{1f61c}", // 😜
    "stuck_out_tongue_closed_eyes" => "\u{1f61d}", // 😝
    "stuck_out_tongue" => "\u{1f61b}", // 😛
    "cry" => "\u{1f622}", // 😢
    "scream" => "\u{1f631}", // 😱
    "sweat" => "\u{1f613}", // 😓
    "joy" => "\u{1f602}", // 😂
    "pensive" => "\u{1f614}", // 😔
    "confounded" => "\u{1f616}", // 😖
    "relieved" => "\u{1f60c}", // 😌
    "sunglasses" => "\u{1f60e}", // 😎
    "sleeping" => "\u{1f634}", // 😴
    "astonished" => "\u{1f632}", // 😲
    "worried" => "\u{1f61f}", // 😟
    "frowning" => "\u{1f626}", // 😦
    "anguished" => "\u{1f627}", // 😧
    "grimacing" => "\u{1f62c}", // 😬
    "open_mouth" => "\u{1f62e}", // 😮
    "neutral_face" => "\u{1f610}", // 😐
    "expressionless" => "\u{1f611}", // 😑
    "hushed" => "\u{1f62f}", // 😯
    "sleepy" => "\u{1f62a}", // 😪
    "no_mouth" => "\u{1f636}", // 😶
    "innocent" => "\u{1f607}", // 😇
    "smirk" => "\u{1f60f}", // 😏
    "unamused" => "\u{1f612}", // 😒
    "star-struck" => "\u{1f929}", // 🤩
    "thinking" => "\u{1f914}", // 🤔
    "face_with_raised_eyebrow" => "\u{1f928}", // 🤨
    "shushing_face" => "\u{1f92b}", // 🤫
    "money_mouth_face" => "\u{1f911}", // 🤑
    "face_with_cowboy_hat" => "\u{1f920}", // 🤠
    "drooling_face" => "\u{1f924}", // 🤤
    "face_with_symbols_on_mouth" => "\u{1f92c}", // 🤬
    "exploding_head" => "\u{1f92f}", // 🤯
    "hot_face" => "\u{1f975}", // 🥵
    "cold_face" => "\u{1f976}", // 🥶
    "woozy_face" => "\u{1f974}", // 🥴
    "partying_face" => "\u{1f973}", // 🥳
    "nauseated_face" => "\u{1f922}", // 🤢
    "face_vomiting" => "\u{1f92e}", // 🤮
    "mask" => "\u{1f637}", // 😷
    "face_with_thermometer" => "\u{1f912}", // 🤒
    "face_with_head_bandage" => "\u{1f915}", // 🤕
    "sneezing_face" => "\u{1f927}", // 🤧
    "robot_face" => "\u{1f916}", // 🤖
    "smiling_imp" => "\u{1f608}", // 😈
    "imp" => "\u{1f47f}", // 👿
    "skull" => "\u{1f480}", // 💀
    "ghost" => "\u{1f47b}", // 👻
    "alien" => "\u{1f47d}", // 👽
    "space_invader" => "\u{1f47e}", // 👾
    "poop" => "\u{1f4a9}", // 💩
    "clown_face" => "\u{1f921}", // 🤡
    "japanese_ogre" => "\u{1f479}", // 👹
    "japanese_goblin" => "\u{1f47a}", // 👺
    "skull_and_crossbones" => "\u{2620}", // ☠
    "pirate_flag" => "\u{1f3f4}\u{200d}\u{2620}", // 🏴‍☠️
};

pub fn get_emoji(key: &str) -> &str {
    EMOJIS.get(key).unwrap_or(&"")
}