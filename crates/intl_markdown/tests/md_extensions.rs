//! Tests for Markdown syntax extensions, specifically hooks (`$[]()`), unsafe variables (`!!{}!!`),
//! and strikethroughs (a la GFM, `~~deleted~~`).

mod harness;

/// Chinese and Japanese content usually do _not_ include spaces between formatted and unformatted
/// segments of a single phrase, such as `**{value}**件の投稿`. But this is technically not valid
/// `strong` formatting according to the CommonMark spec, since the right flank of the ending
/// delimiter is a non-space Unicode character.
///
/// See more information in the CommonMark discussion here:
/// https://talk.commonmark.org/t/emphasis-and-east-asian-text/2491/5
/// https://github.com/commonmark/cmark/pull/208
///
/// Because this library is explicitly intended to support many languages including most Asian
/// languages, we are adding an extension to the Markdown rules to accommodate these situations.
/// The following tests assert that the special cases for East Asian languages function in a
/// logically-similar way to Western languages.
mod asian_punctuation {
    use crate::harness::{ast_test, icu_string_test, run_icu_string_test};
    icu_string_test!(
        japanese_adjacent_formatting,
        "**{value}**件の投稿",
        r#"<b>{value}</b>件の投稿"#
    );
    icu_string_test!(
        japanese_spaced_formatting,
        "**{value}** 件の投稿",
        r#"<b>{value}</b> 件の投稿"#
    );
    icu_string_test!(
        korean_western_punctuation,
        "*스크립트(script)*라고",
        r#"<i>스크립트(script)</i>라고"#
    );
    icu_string_test!(
        japanese_adjacent_non_punctuation_emphasis,
        "バーを*{numSubscriptions}*回ブーストしました！",
        r#"バーを<i>{numSubscriptions}</i>回ブーストしました！"#
    );
    icu_string_test!(
        japanese_adjacent_non_punctuation_double_emphasis,
        "バーを**{numSubscriptions}**回ブーストしました！",
        r#"バーを<b>{numSubscriptions}</b>回ブーストしました！"#
    );

    use test_case::test_case;
    // Test cases taken from https://github.com/tats-u/markdown-cjk-friendly/blob/main/testcases
    #[test_case("a~~a()~~あ","a<del>a()</del>あ"; "trailing_japanese")]
    #[test_case("あ~~()a~~a","あ<del>()a</del>a"; "leading_japanese")]
    #[test_case("𩸽~~()a~~a","𩸽<del>()a</del>a"; "leading_supplementary_ideograph")]
    #[test_case("a~~a()~~𩸽","a<del>a()</del>𩸽"; "trailing_supplementary_ideograph")]
    #[test_case("葛~~()a~~a","葛<del>()a</del>a"; "leading_basic_multilingual_ideograph")]
    #[test_case("a~~()a~~葛","a<del>()a</del>葛"; "trailing_basic_multilingual_ideograph")]
    #[test_case("羽︀~~()a~~a","羽︀<del>()a</del>a"; "leading_basic_multilingual_ideograph_2")]
    #[test_case("a~~()a~~羽︀","a<del>()a</del>羽︀"; "trailing_basic_multilingual_ideograph_2")]
    #[test_case("a~~「a~~」","a<del>「a</del>」"; "trailing_quote_punctuation")]
    #[test_case("「~~a」~~a","「<del>a」</del>a"; "leading_quote_punctuation")]
    #[test_case("~~a~~：~~a~~","<del>a</del>：<del>a</del>"; "inner_punctuation")]
    fn gfm_strikethrough(input: &str, output: &str) {
        run_icu_string_test(input, output, false);
    }

    ast_test!(
        korean_nested_link,
        "**이 [링크](https://example.kr/)**만을 강조하고 싶다.",
        r#"[[8,"$b",["이 ",[8,"$link",["링크"],["https://example.kr/"]]]],"만을 강조하고 싶다."]"#
    );
    ast_test!(
        korean_nested_punctuation,
        "**스크립트(script)**라고",
        r#"[[8,"$b",["스크립트(script)"]],"라고"]"#
    );
    ast_test!(
        korean_nested_code_span,
        "패키지를 발행하려면 **`npm publish`**를 실행하십시오.",
        r#"["패키지를 발행하려면 ",[8,"$b",[[8,"$code",["npm publish"]]]],"를 실행하십시오."]"#
    );
    ast_test!(
        korean_leading_emphasis,
        "**안녕(hello)**하세요.",
        r#"[[8,"$b",["안녕(hello)"]],"하세요."]"#
    );
    ast_test!(
        korean_trailing_emphasis,
        "ᅡ**(a)**",
        r#"["ᅡ",[8,"$b",["(a)"]]]"#
    );
    ast_test!(
        korean_trailing_word,
        "**(k)**ᄏ",
        r#"[[8,"$b",["(k)"]],"ᄏ"]"#
    );

    #[test_case("a**〰**a", "a<b>〰</b>a"; "wavy_dash")]
    #[test_case("a**〽**a", "a<b>〽</b>a"; "part_alternation_mark")]
    #[test_case("a**🈂**a", "a<b>🈂</b>a"; "squared_katakana_sa")]
    #[test_case("a**🈷**a", "a<b>🈷</b>a"; "squared_cjk_ideograph")]
    #[test_case("a**㊗**a", "a<b>㊗</b>a"; "circled_ideograph_congraulation")]
    #[test_case("a**㊙**a", "a<b>㊙</b>a"; "circled_ideograph_secret")]
    fn pseudo_emoji(input: &str, output: &str) {
        run_icu_string_test(input, output, false);
    }

    icu_string_test!(
        test_0,
        "これは**私のやりたかったこと。**だからするの。",
        "これは<b>私のやりたかったこと。</b>だからするの。"
    );
    icu_string_test!(
        test_1,
        "これは**私のやりたかったこと。**だからするの。",
        "これは<b>私のやりたかったこと。</b>だからするの。"
    );
    icu_string_test!(
        test_1a,
        "これは*私のやりたかったこと。*だからするの。",
        "これは<i>私のやりたかったこと。</i>だからするの。"
    );
    ast_test!(
        test_2,
        "**[製品ほげ](./product-foo)**と**[製品ふが](./product-bar)**をお試しください",
        r#"[[8,"$b",[[8,"$link",["製品ほげ"],["./product-foo"]]]],"と",[8,"$b",[[8,"$link",["製品ふが"],["./product-bar"]]]],"をお試しください"]"#
    );
    ast_test!(
        test_3,
        "単語と**[単語と](word-and)**単語",
        r#"["単語と",[8,"$b",[[8,"$link",["単語と"],["word-and"]]]],"単語"]"#
    );
    icu_string_test!(
        test_4,
        "**これは太字になりません。**ご注意ください。",
        "<b>これは太字になりません。</b>ご注意ください。"
    );
    icu_string_test!(
        test_5,
        "カッコに注意**（太字にならない）**文が続く場合に要警戒。",
        "カッコに注意<b>（太字にならない）</b>文が続く場合に要警戒。"
    );
    ast_test!(
        test_6,
        "**[リンク](https://example.com)**も注意。（画像も同様）",
        r#"[[8,"$b",[[8,"$link",["リンク"],["https://example.com"]]]],"も注意。（画像も同様）"]"#
    );
    icu_string_test!(
        test_7,
        r#"先頭の**\`コード\`も注意。**"#,
        r#"先頭の<b>`コード`も注意。</b>"#
    );
    icu_string_test!(
        test_8,
        r#"**末尾の\`コード\`**も注意。"#,
        r#"<b>末尾の`コード`</b>も注意。"#
    );
    icu_string_test!(
        test_9,
        "税込**¥10,000**で入手できます。",
        "税込<b>¥10,000</b>で入手できます。"
    );
    icu_string_test!(test_10, "正解は**④**です。", "正解は<b>④</b>です。");
    icu_string_test!(
        test_11,
        "太郎は**「こんにちわ」**といった",
        "太郎は<b>「こんにちわ」</b>といった"
    );
    icu_string_test!(
        test_12,
        r#"太郎は**"こんにちわ"**といった"#,
        r#"太郎は<b>&quot;こんにちわ&quot;</b>といった"#
    );
    icu_string_test!(
        test_13,
        r#"太郎は**こんにちわ**といった"#,
        r#"太郎は<b>こんにちわ</b>といった"#
    );
    icu_string_test!(
        test_14,
        r#"太郎は**「Hello」**といった"#,
        r#"太郎は<b>「Hello」</b>といった"#
    );
    icu_string_test!(
        test_14a,
        r#"太郎は*「Hello」*といった"#,
        r#"太郎は<i>「Hello」</i>といった"#
    );
    icu_string_test!(
        test_15,
        r#"太郎は**"Hello"**といった"#,
        r#"太郎は<b>&quot;Hello&quot;</b>といった"#
    );
    icu_string_test!(
        test_16,
        r#"太郎は**Hello**といった"#,
        r#"太郎は<b>Hello</b>といった"#
    );
    icu_string_test!(
        test_17,
        r#"太郎は**「Oh my god」**といった"#,
        r#"太郎は<b>「Oh my god」</b>といった"#
    );
    icu_string_test!(
        test_18,
        r#"太郎は**"Oh my god"**といった"#,
        r#"太郎は<b>&quot;Oh my god&quot;</b>といった"#
    );
    icu_string_test!(
        test_19,
        "太郎は**Oh my god**といった",
        "太郎は<b>Oh my god</b>といった"
    );
    icu_string_test!(
        test_20,
        "**C#**や**F#**は**「.NET」**というプラットフォーム上で動作します。",
        "<b>C#</b>や<b>F#</b>は<b>「.NET」</b>というプラットフォーム上で動作します。"
    );
    icu_string_test!(
        test_21,
        "IDが**001号**になります。",
        "IDが<b>001号</b>になります。"
    );
    icu_string_test!(
        test_22,
        "IDが**００１号**になります。",
        "IDが<b>００１号</b>になります。"
    );
    icu_string_test!(
        test_23,
        "Go**「初心者」**を対象とした記事です。",
        "Go<b>「初心者」</b>を対象とした記事です。"
    );
    ast_test!(
        test_24,
        "**[リンク](https://example.com)**も注意。",
        r#"[[8,"$b",[[8,"$link",["リンク"],["https://example.com"]]]],"も注意。"]"#
    );
    icu_string_test!(
        test_25,
        "**⻲田太郎**と申します",
        "<b>⻲田太郎</b>と申します"
    );
    icu_string_test!(test_26, "・**㋐**:選択肢１つ目", "・<b>㋐</b>:選択肢１つ目");
    icu_string_test!(test_27, "**真，**她", "<b>真，</b>她");
    icu_string_test!(test_28, "**真。**她", "<b>真。</b>她");
    icu_string_test!(test_29, "**真、**她", "<b>真、</b>她");
    icu_string_test!(test_30, "**真；**她", "<b>真；</b>她");
    icu_string_test!(test_31, "**真：**她", "<b>真：</b>她");
    icu_string_test!(test_32, "**真？**她", "<b>真？</b>她");
    icu_string_test!(test_33, "**真！**她", "<b>真！</b>她");
    icu_string_test!(test_34, "**真“**她", "<b>真“</b>她");
    icu_string_test!(test_35, "**真”**她", "<b>真”</b>她");
    icu_string_test!(test_36, "**真‘**她", "<b>真‘</b>她");
    icu_string_test!(test_37, "**真’**她", "<b>真’</b>她");
    icu_string_test!(test_38, "**真（**她", "<b>真（</b>她");
    icu_string_test!(test_39, "真**（她**", "真<b>（她</b>");
    icu_string_test!(test_40, "**真）**她", "<b>真）</b>她");
    icu_string_test!(test_41, "**真【**她", "<b>真【</b>她");
    icu_string_test!(test_42, "真**【她**", "真<b>【她</b>");
    icu_string_test!(test_43, "**真】**她", "<b>真】</b>她");
    icu_string_test!(test_44, "**真《**她", "<b>真《</b>她");
    icu_string_test!(test_45, "真**《她**", "真<b>《她</b>");
    icu_string_test!(test_46, "**真》**她", "<b>真》</b>她");
    icu_string_test!(test_47, "**真—**她", "<b>真—</b>她");
    icu_string_test!(test_48, "**真～**她", "<b>真～</b>她");
    icu_string_test!(test_49, "**真…**她", "<b>真…</b>她");
    icu_string_test!(test_50, "**真·**她", "<b>真·</b>她");
    icu_string_test!(test_51, "**真〃**她", "<b>真〃</b>她");
    icu_string_test!(test_52, "**真-**她", "<b>真-</b>她");
    icu_string_test!(test_53, "**真々**她", "<b>真々</b>她");
    icu_string_test!(test_54, "**真**她", "<b>真</b>她");
    icu_string_test!(test_55, "**真，** 她", "<b>真，</b> 她");
    icu_string_test!(test_56, "**真**，她", "<b>真</b>，她");
    icu_string_test!(
        test_57,
        "**真，**&ZeroWidthSpace;她",
        "<b>真，</b>\u{200b}她"
    );
    icu_string_test!(
        test_58,
        "私は**⻲田太郎**と申します",
        "私は<b>⻲田太郎</b>と申します"
    );
    icu_string_test!(
        test_59,
        "選択肢**㋐**: 1つ目の選択肢",
        "選択肢<b>㋐</b>: 1つ目の選択肢"
    );
    icu_string_test!(
        test_60,
        "**さようなら︙**と太郎はいった。",
        "<b>さようなら︙</b>と太郎はいった。"
    );
    icu_string_test!(
        test_61,
        ".NET**（.NET Frameworkは不可）**では、",
        ".NET<b>（.NET Frameworkは不可）</b>では、"
    );
    icu_string_test!(
        test_62,
        "「禰󠄀」の偏は示ではなく**礻**です。",
        "「禰󠄀」の偏は示ではなく<b>礻</b>です。"
    );
    icu_string_test!(
        test_63,
        "Git**（注：不是GitHub）**",
        "Git<b>（注：不是GitHub）</b>"
    );
    icu_string_test!(
        test_64,
        "太郎は**「こんにちわ」**といった。",
        "太郎は<b>「こんにちわ」</b>といった。"
    );
    icu_string_test!(
        test_65,
        "𰻞𰻞**（ビャンビャン）**麺",
        "𰻞𰻞<b>（ビャンビャン）</b>麺"
    );
    icu_string_test!(
        test_66,
        "𰻞𰻞**(ビャンビャン)**麺",
        "𰻞𰻞<b>(ビャンビャン)</b>麺"
    );
    icu_string_test!(
        test_67,
        "ハイパーテキストコーヒーポット制御プロトコル**(HTCPCP)**",
        "ハイパーテキストコーヒーポット制御プロトコル<b>(HTCPCP)</b>"
    );
    icu_string_test!(test_68, "﨑**(崎)**", "﨑<b>(崎)</b>");
    ast_test!(
        test_69,
        "国際規格**[ECMA-262](https://tc39.es/ecma262/)**",
        r#"["国際規格",[8,"$b",[[8,"$link",["ECMA-262"],["https://tc39.es/ecma262/"]]]]]"#
    );
    icu_string_test!(test_70, "㐧**(第の俗字)**", "㐧<b>(第の俗字)</b>");
    icu_string_test!(
        test_71,
        "𠮟**(こちらが正式表記)**",
        "𠮟<b>(こちらが正式表記)</b>"
    );
    icu_string_test!(
        test_72,
        "𪜈**(トモの合略仮名)**",
        "𪜈<b>(トモの合略仮名)</b>"
    );
    icu_string_test!(test_73, "𫠉**(馬の俗字)**", "𫠉<b>(馬の俗字)</b>");
    icu_string_test!(
        test_74,
        "谺𬤲**(こだま)**石神社",
        "谺𬤲<b>(こだま)</b>石神社"
    );
    icu_string_test!(test_75, "石𮧟**(いしただら)**", "石𮧟<b>(いしただら)</b>");
    icu_string_test!(
        test_76,
        "**推荐几个框架：**React、Vue等前端框架。",
        "<b>推荐几个框架：</b>React、Vue等前端框架。"
    );
    icu_string_test!(
        test_77,
        "葛󠄀**(こちらが正式表記)**城市",
        "葛󠄀<b>(こちらが正式表記)</b>城市"
    );
    icu_string_test!(
        test_78,
        "禰󠄀**(こちらが正式表記)**豆子",
        "禰󠄀<b>(こちらが正式表記)</b>豆子"
    );
    icu_string_test!(test_79, "𱟛**(U+317DB)**", "𱟛<b>(U+317DB)</b>");
    icu_string_test!(
        test_80,
        "阿寒湖アイヌシアターイコㇿ**(Akanko Ainu Theater Ikor)**",
        "阿寒湖アイヌシアターイコㇿ<b>(Akanko Ainu Theater Ikor)</b>"
    );
    icu_string_test!(test_81, "あ𛀙**(か)**よろし", "あ𛀙<b>(か)</b>よろし");
    icu_string_test!(
        test_82,
        "𮹝**(simplified form of 龘 in China)**",
        "𮹝<b>(simplified form of 龘 in China)</b>"
    );
    icu_string_test!(
        test_83,
        "大塚︀**(U+585A U+FE00)** 大塚**(U+FA10)**",
        "大塚︀<b>(U+585A U+FE00)</b> 大塚<b>(U+FA10)</b>"
    );
    icu_string_test!(test_84, "〽︎**(庵点)**は、", "〽︎<b>(庵点)</b>は、");
    icu_string_test!(test_85, "**“︁Git”︁**Hub", "<b>“︁Git”︁</b>Hub");

    icu_string_test!(underscore_1, "__注意__：注意事項", "<b>注意</b>：注意事項");
    icu_string_test!(underscore_2, "注意：__注意事項__", "注意：<b>注意事項</b>");
    icu_string_test!(
        underscore_3,
        "正體字。︁_Traditional._",
        "正體字。︁<i>Traditional.</i>"
    );
    icu_string_test!(
        underscore_4,
        "正體字。︁__Hong Kong and Taiwan.__",
        "正體字。︁<b>Hong Kong and Taiwan.</b>"
    );
    icu_string_test!(
        underscore_5,
        "简体字 / 新字体。︀_Simplified._",
        "简体字 / 新字体。︀<i>Simplified.</i>"
    );
    icu_string_test!(
        underscore_6,
        "简体字 / 新字体。︀__Mainland China or Japan.__",
        "简体字 / 新字体。︀<b>Mainland China or Japan.</b>"
    );
    icu_string_test!(underscore_7, "“︁Git”︁__Hub__", "“︁Git”︁<b>Hub</b>");
}

mod hooks {
    use crate::harness::ast_test;
    ast_test!(
        basic_hook,
        "$[text](someHook)",
        r#"[[8,"someHook",["text"]]]"#
    );
    ast_test!(empty_hook, "$[](someHook)", r#"[[8,"someHook",[]]]"#);
    ast_test!(
        link_inside_hook,
        "$[text [link](./foo)](someHook)",
        r#"[[8,"someHook",["text ",[8,"$link",["link"],["./foo"]]]]]"#
    );
    ast_test!(
        hook_inside_link,
        "[link $[text](someHook)](./foo)",
        r#"[[8,"$link",["link ",[8,"someHook",["text"]]],["./foo"]]]"#
    );
    ast_test!(
        hook_inside_hook,
        "$[outer $[inner](hook1)](hook2)",
        r#"[[8,"hook2",["outer ",[8,"hook1",["inner"]]]]]"#
    );
    ast_test!(
        disallow_dynamic_hook,
        "$[inner]({target})",
        r#"["$[inner](",[1,"target"],")"]"#
    );
    ast_test!(
        allow_dynamic_content,
        "$[{target}](someHook)",
        r#"[[8,"someHook",[[1,"target"]]]]"#
    );
}

mod unsafe_variable_strings {
    use crate::harness::ast_test;
    ast_test!(basic_unsafe, "!!{username}!!", r#"[[1,"username"]]"#);
    ast_test!(
        nested_unsafe,
        "{count, plural, one {hi !!{username}!!}}",
        r#"[[6,"count",{"one":["hi ",[1,"username"]]},0,"cardinal"]]"#
    );
    ast_test!(
        wrapped_unsafe,
        "**!!{username}!!**",
        r#"[[8,"$b",[[1,"username"]]]]"#
    );
}

mod strikethrough {
    use crate::harness::icu_string_test;
    icu_string_test!(basic_strikethrough, "~one tilde~", "<del>one tilde</del>");
    icu_string_test!(
        double_strikethrough,
        "~~two tildes~~",
        "<del>two tildes</del>"
    );
    icu_string_test!(
        too_many,
        "~~~not strikethrough~~~",
        "~~~not strikethrough~~~"
    );
    icu_string_test!(
        intra_word,
        "intra~~word~~strike",
        "intra<del>word</del>strike"
    );
    icu_string_test!(
        matched_intra_word,
        "~~intra~~word~~strike~~",
        "<del>intra</del>word<del>strike</del>"
    );
    icu_string_test!(no_mixed, "~~no mixed~", "~~no mixed~");
    icu_string_test!(
        no_partial_usage,
        "~~~can't use part of a run~~",
        "~~~can't use part of a run~~"
    );
    icu_string_test!(no_single_run, "~~~", "~~~");
    icu_string_test!(
        first_wins,
        "~~first ~wins~~ easy~",
        "<del>first ~wins</del> easy~"
    );
    icu_string_test!(
        nesting,
        "~~nesting ~works~ with bounds~~",
        "<del>nesting <del>works</del> with bounds</del>"
    );
    icu_string_test!(
        no_direct_nesting,
        "~~~direct nesting ~doesn't work~~~",
        "~~~direct nesting ~doesn't work~~~"
    );
    icu_string_test!(
        no_boundary_crossing,
        "~~no *boundary~~ crossing*",
        "<del>no *boundary</del> crossing*"
    );
    icu_string_test!(
        no_reverse_boundary_crossing,
        "*no ~boundary* crossing~",
        "<i>no ~boundary</i> crossing~"
    );
    icu_string_test!(
        escaped,
        "~~this is \\~\\~escaped~~",
        "<del>this is ~~escaped</del>"
    );
    icu_string_test!(escaped_open, "\\~this is escaped~", "~this is escaped~");
    icu_string_test!(escaped_close, "~this is escaped\\~", "~this is escaped~");
    icu_string_test!(
        escaped_split,
        "~\\~this is escaped~~",
        "~~this is escaped~~"
    );
    icu_string_test!(
        escaped_leading,
        "\\~~this is escaped~~",
        "~~this is escaped~~"
    );
    icu_string_test!(
        escaped_trailing,
        "~~this is escaped~\\~",
        "~~this is escaped~~"
    );
    icu_string_test!(
        escaped_matches_single,
        "\\~~this is escaped~",
        "~<del>this is escaped</del>"
    );
    icu_string_test!(
        punctuation_flanking_double,
        "flanked punctuation~~!~~",
        "flanked punctuation<del>!</del>"
    );
    icu_string_test!(
        punctuation_flanking_single,
        "flanked punctuation single~!~",
        "flanked punctuation single~!~"
    );
}
