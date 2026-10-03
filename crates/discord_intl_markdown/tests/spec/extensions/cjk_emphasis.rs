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
use crate::spec::extensions::harness;

#[test]
fn japanese_adjacent_formatting() {
    let input = "**{value}**件の投稿";
    let expected = r#"<strong>{value}</strong>件の投稿"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn japanese_spaced_formatting() {
    let input = "**{value}** 件の投稿";
    let expected = r#"<strong>{value}</strong> 件の投稿"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn korean_western_punctuation() {
    let input = "*스크립트(script)*라고";
    let expected = r#"<em>스크립트(script)</em>라고"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn japanese_adjacent_non_punctuation_emphasis() {
    let input = "バーを*{numSubscriptions}*回ブーストしました！";
    let expected = r#"バーを<em>{numSubscriptions}</em>回ブーストしました！"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn japanese_adjacent_non_punctuation_double_emphasis() {
    let input = "バーを**{numSubscriptions}**回ブーストしました！";
    let expected = r#"バーを<strong>{numSubscriptions}</strong>回ブーストしました！"#;

    assert_eq!(expected, harness::parse_inline(input));
}

// Strikethrough Test cases taken from https://github.com/tats-u/markdown-cjk-friendly/blob/main/testcases
#[test]
fn strikethrough_trailing_japanese() {
    let input = "a~~a()~~あ";
    let expected = "a<del>a()</del>あ";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn strikethrough_leading_japanese() {
    let input = "あ~~()a~~a";
    let expected = "あ<del>()a</del>a";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn strikethrough_leading_supplementary_ideograph() {
    let input = "𩸽~~()a~~a";
    let expected = "𩸽<del>()a</del>a";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn strikethrough_trailing_supplementary_ideograph() {
    let input = "a~~a()~~𩸽";
    let expected = "a<del>a()</del>𩸽";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn strikethrough_leading_basic_multilingual_ideograph() {
    let input = "葛~~()a~~a";
    let expected = "葛<del>()a</del>a";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn strikethrough_trailing_basic_multilingual_ideograph() {
    let input = "a~~()a~~葛";
    let expected = "a<del>()a</del>葛";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn strikethrough_leading_basic_multilingual_ideograph_2() {
    let input = "羽︀~~()a~~a";
    let expected = "羽︀<del>()a</del>a";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn strikethrough_trailing_basic_multilingual_ideograph_2() {
    let input = "a~~()a~~羽︀";
    let expected = "a<del>()a</del>羽︀";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn strikethrough_trailing_quote_punctuation() {
    let input = "a~~「a~~」";
    let expected = "a<del>「a</del>」";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn strikethrough_leading_quote_punctuation() {
    let input = "「~~a」~~a";
    let expected = "「<del>a」</del>a";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn strikethrough_inner_punctuation() {
    let input = "~~a~~：~~a~~";
    let expected = "<del>a</del>：<del>a</del>";

    assert_eq!(expected, harness::parse_inline(input));
}

#[test]
fn korean_nested_link() {
    let input = "**이 [링크](https://example.kr/)**만을 강조하고 싶다.";
    let expected =
        r#"<strong>이 <a href="https://example.kr/">링크</a></strong>만을 강조하고 싶다."#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn korean_nested_punctuation() {
    let input = "**스크립트(script)**라고";
    let expected = "<strong>스크립트(script)</strong>라고";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn korean_nested_code_span() {
    let input = "패키지를 발행하려면 **`npm publish`**를 실행하십시오.";
    let expected = "패키지를 발행하려면 <strong><code>npm publish</code></strong>를 실행하십시오.";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn korean_leading_emphasis() {
    let input = "**안녕(hello)**하세요.";
    let expected = "<strong>안녕(hello)</strong>하세요.";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn korean_trailing_emphasis() {
    let input = "ᅡ**(a)**";
    let expected = "ᅡ<strong>(a)</strong>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn korean_trailing_word() {
    let input = "**(k)**ᄏ";
    let expected = "<strong>(k)</strong>ᄏ";

    assert_eq!(expected, harness::parse_inline(input));
}

#[test]
fn pseudo_emoji_wavy_dash() {
    let input = "a**〰**a";
    let expected = "a<strong>〰</strong>a";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn pseudo_emoji_part_alternation_mark() {
    let input = "a**〽**a";
    let expected = "a<strong>〽</strong>a";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn pseudo_emoji_squared_katakana_sa() {
    let input = "a**🈂**a";
    let expected = "a<strong>🈂</strong>a";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn pseudo_emoji_squared_cjk_ideograph() {
    let input = "a**🈷**a";
    let expected = "a<strong>🈷</strong>a";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn pseudo_emoji_circled_ideograph_congratulation() {
    let input = "a**㊗**a";
    let expected = "a<strong>㊗</strong>a";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn pseudo_emoji_circled_ideograph_secret() {
    let input = "a**㊙**a";
    let expected = "a<strong>㊙</strong>a";

    assert_eq!(expected, harness::parse_inline(input));
}

#[test]
fn cjk_test_0() {
    let input = "これは**私のやりたかったこと。**だからするの。";
    let expected = "これは<strong>私のやりたかったこと。</strong>だからするの。";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_1() {
    let input = "これは**私のやりたかったこと。**だからするの。";
    let expected = "これは<strong>私のやりたかったこと。</strong>だからするの。";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_1a() {
    let input = "これは*私のやりたかったこと。*だからするの。";
    let expected = "これは<em>私のやりたかったこと。</em>だからするの。";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_2() {
    let input = "**[製品ほげ](./product-foo)**と**[製品ふが](./product-bar)**をお試しください";
    let expected = r#"<strong><a href="./product-foo">製品ほげ</a></strong>と<strong><a href="./product-bar">製品ふが</a></strong>をお試しください"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_3() {
    let input = "単語と**[単語と](word-and)**単語";
    let expected = r#"単語と<strong><a href="word-and">単語と</a></strong>単語"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_4() {
    let input = "**これは太字になりません。**ご注意ください。";
    let expected = "<strong>これは太字になりません。</strong>ご注意ください。";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_5() {
    let input = "カッコに注意**（太字にならない）**文が続く場合に要警戒。";
    let expected = "カッコに注意<strong>（太字にならない）</strong>文が続く場合に要警戒。";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_6() {
    let input = "**[リンク](https://example.com)**も注意。（画像も同様）";
    let expected =
        r#"<strong><a href="https://example.com">リンク</a></strong>も注意。（画像も同様）"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_7() {
    let input = r#"先頭の**\`コード\`も注意。**"#;
    let expected = r#"先頭の<strong>`コード`も注意。</strong>"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_8() {
    let input = r#"**末尾の\`コード\`**も注意。"#;
    let expected = r#"<strong>末尾の`コード`</strong>も注意。"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_9() {
    let input = "税込**¥10,000**で入手できます。";
    let expected = "税込<strong>¥10,000</strong>で入手できます。";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_10() {
    let input = "正解は**④**です。";
    let expected = "正解は<strong>④</strong>です。";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_11() {
    let input = "太郎は**「こんにちわ」**といった";
    let expected = "太郎は<strong>「こんにちわ」</strong>といった";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_12() {
    let input = r#"太郎は**"こんにちわ"**といった"#;
    let expected = r#"太郎は<strong>&quot;こんにちわ&quot;</strong>といった"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_13() {
    let input = r#"太郎は**こんにちわ**といった"#;
    let expected = r#"太郎は<strong>こんにちわ</strong>といった"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_14() {
    let input = r#"太郎は**「Hello」**といった"#;
    let expected = r#"太郎は<strong>「Hello」</strong>といった"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_14a() {
    let input = r#"太郎は*「Hello」*といった"#;
    let expected = r#"太郎は<em>「Hello」</em>といった"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_15() {
    let input = r#"太郎は**"Hello"**といった"#;
    let expected = r#"太郎は<strong>&quot;Hello&quot;</strong>といった"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_16() {
    let input = r#"太郎は**Hello**といった"#;
    let expected = r#"太郎は<strong>Hello</strong>といった"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_17() {
    let input = r#"太郎は**「Oh my god」**といった"#;
    let expected = r#"太郎は<strong>「Oh my god」</strong>といった"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_18() {
    let input = r#"太郎は**"Oh my god"**といった"#;
    let expected = r#"太郎は<strong>&quot;Oh my god&quot;</strong>といった"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_19() {
    let input = "太郎は**Oh my god**といった";
    let expected = "太郎は<strong>Oh my god</strong>といった";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_20() {
    let input = "**C#**や**F#**は**「.NET」**というプラットフォーム上で動作します。";
    let expected = "<strong>C#</strong>や<strong>F#</strong>は<strong>「.NET」</strong>というプラットフォーム上で動作します。";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_21() {
    let input = "IDが**001号**になります。";
    let expected = "IDが<strong>001号</strong>になります。";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_22() {
    let input = "IDが**００１号**になります。";
    let expected = "IDが<strong>００１号</strong>になります。";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_23() {
    let input = "Go**「初心者」**を対象とした記事です。";
    let expected = "Go<strong>「初心者」</strong>を対象とした記事です。";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_24() {
    let input = "**[リンク](https://example.com)**も注意。";
    let expected = r#"<strong><a href="https://example.com">リンク</a></strong>も注意。"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_25() {
    let input = "**⻲田太郎**と申します";
    let expected = "<strong>⻲田太郎</strong>と申します";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_26() {
    let input = "・**㋐**:選択肢１つ目";
    let expected = "・<strong>㋐</strong>:選択肢１つ目";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_27() {
    let input = "**真，**她";
    let expected = "<strong>真，</strong>她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_28() {
    let input = "**真。**她";
    let expected = "<strong>真。</strong>她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_29() {
    let input = "**真、**她";
    let expected = "<strong>真、</strong>她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_30() {
    let input = "**真；**她";
    let expected = "<strong>真；</strong>她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_31() {
    let input = "**真：**她";
    let expected = "<strong>真：</strong>她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_32() {
    let input = "**真？**她";
    let expected = "<strong>真？</strong>她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_33() {
    let input = "**真！**她";
    let expected = "<strong>真！</strong>她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_34() {
    let input = "**真“**她";
    let expected = "<strong>真“</strong>她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_35() {
    let input = "**真”**她";
    let expected = "<strong>真”</strong>她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_36() {
    let input = "**真‘**她";
    let expected = "<strong>真‘</strong>她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_37() {
    let input = "**真’**她";
    let expected = "<strong>真’</strong>她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_38() {
    let input = "**真（**她";
    let expected = "<strong>真（</strong>她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_39() {
    let input = "真**（她**";
    let expected = "真<strong>（她</strong>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_40() {
    let input = "**真）**她";
    let expected = "<strong>真）</strong>她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_41() {
    let input = "**真【**她";
    let expected = "<strong>真【</strong>她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_42() {
    let input = "真**【她**";
    let expected = "真<strong>【她</strong>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_43() {
    let input = "**真】**她";
    let expected = "<strong>真】</strong>她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_44() {
    let input = "**真《**她";
    let expected = "<strong>真《</strong>她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_45() {
    let input = "真**《她**";
    let expected = "真<strong>《她</strong>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_46() {
    let input = "**真》**她";
    let expected = "<strong>真》</strong>她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_47() {
    let input = "**真—**她";
    let expected = "<strong>真—</strong>她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_48() {
    let input = "**真～**她";
    let expected = "<strong>真～</strong>她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_49() {
    let input = "**真…**她";
    let expected = "<strong>真…</strong>她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_50() {
    let input = "**真·**她";
    let expected = "<strong>真·</strong>她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_51() {
    let input = "**真〃**她";
    let expected = "<strong>真〃</strong>她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_52() {
    let input = "**真-**她";
    let expected = "<strong>真-</strong>她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_53() {
    let input = "**真々**她";
    let expected = "<strong>真々</strong>她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_54() {
    let input = "**真**她";
    let expected = "<strong>真</strong>她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_55() {
    let input = "**真，** 她";
    let expected = "<strong>真，</strong> 她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_56() {
    let input = "**真**，她";
    let expected = "<strong>真</strong>，她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_57() {
    let input = "**真，**&ZeroWidthSpace;她";
    let expected = "<strong>真，</strong>\u{200b}她";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_58() {
    let input = "私は**⻲田太郎**と申します";
    let expected = "私は<strong>⻲田太郎</strong>と申します";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_59() {
    let input = "選択肢**㋐**: 1つ目の選択肢";
    let expected = "選択肢<strong>㋐</strong>: 1つ目の選択肢";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_60() {
    let input = "**さようなら︙**と太郎はいった。";
    let expected = "<strong>さようなら︙</strong>と太郎はいった。";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_61() {
    let input = ".NET**（.NET Frameworkは不可）**では、";
    let expected = ".NET<strong>（.NET Frameworkは不可）</strong>では、";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_62() {
    let input = "「禰󠄀」の偏は示ではなく**礻**です。";
    let expected = "「禰󠄀」の偏は示ではなく<strong>礻</strong>です。";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_63() {
    let input = "Git**（注：不是GitHub）**";
    let expected = "Git<strong>（注：不是GitHub）</strong>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_64() {
    let input = "太郎は**「こんにちわ」**といった。";
    let expected = "太郎は<strong>「こんにちわ」</strong>といった。";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_65() {
    let input = "𰻞𰻞**（ビャンビャン）**麺";
    let expected = "𰻞𰻞<strong>（ビャンビャン）</strong>麺";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_66() {
    let input = "𰻞𰻞**(ビャンビャン)**麺";
    let expected = "𰻞𰻞<strong>(ビャンビャン)</strong>麺";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_67() {
    let input = "ハイパーテキストコーヒーポット制御プロトコル**(HTCPCP)**";
    let expected = "ハイパーテキストコーヒーポット制御プロトコル<strong>(HTCPCP)</strong>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_68() {
    let input = "﨑**(崎)**";
    let expected = "﨑<strong>(崎)</strong>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_69() {
    let input = "国際規格**[ECMA-262](https://tc39.es/ecma262/)**";
    let expected = r#"国際規格<strong><a href="https://tc39.es/ecma262/">ECMA-262</a></strong>"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_70() {
    let input = "㐧**(第の俗字)**";
    let expected = "㐧<strong>(第の俗字)</strong>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_71() {
    let input = "𠮟**(こちらが正式表記)**";
    let expected = "𠮟<strong>(こちらが正式表記)</strong>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_72() {
    let input = "𪜈**(トモの合略仮名)**";
    let expected = "𪜈<strong>(トモの合略仮名)</strong>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_73() {
    let input = "𫠉**(馬の俗字)**";
    let expected = "𫠉<strong>(馬の俗字)</strong>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_74() {
    let input = "谺𬤲**(こだま)**石神社";
    let expected = "谺𬤲<strong>(こだま)</strong>石神社";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_75() {
    let input = "石𮧟**(いしただら)**";
    let expected = "石𮧟<strong>(いしただら)</strong>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_76() {
    let input = "**推荐几个框架：**React、Vue等前端框架。";
    let expected = "<strong>推荐几个框架：</strong>React、Vue等前端框架。";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_77() {
    let input = "葛󠄀**(こちらが正式表記)**城市";
    let expected = "葛󠄀<strong>(こちらが正式表記)</strong>城市";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_78() {
    let input = "禰󠄀**(こちらが正式表記)**豆子";
    let expected = "禰󠄀<strong>(こちらが正式表記)</strong>豆子";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_79() {
    let input = "𱟛**(U+317DB)**";
    let expected = "𱟛<strong>(U+317DB)</strong>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_80() {
    let input = "阿寒湖アイヌシアターイコㇿ**(Akanko Ainu Theater Ikor)**";
    let expected = "阿寒湖アイヌシアターイコㇿ<strong>(Akanko Ainu Theater Ikor)</strong>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_81() {
    let input = "あ𛀙**(か)**よろし";
    let expected = "あ𛀙<strong>(か)</strong>よろし";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_82() {
    let input = "𮹝**(simplified form of 龘 in China)**";
    let expected = "𮹝<strong>(simplified form of 龘 in China)</strong>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_83() {
    let input = "大塚︀**(U+585A U+FE00)** 大塚**(U+FA10)**";
    let expected = "大塚︀<strong>(U+585A U+FE00)</strong> 大塚<strong>(U+FA10)</strong>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_84() {
    let input = "〽︎**(庵点)**は、";
    let expected = "〽︎<strong>(庵点)</strong>は、";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_test_85() {
    let input = "**“︁Git”︁**Hub";
    let expected = "<strong>“︁Git”︁</strong>Hub";

    assert_eq!(expected, harness::parse_inline(input));
}

#[test]
fn cjk_underscore_1() {
    let input = "__注意__：注意事項";
    let expected = "<strong>注意</strong>：注意事項";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_underscore_2() {
    let input = "注意：__注意事項__";
    let expected = "注意：<strong>注意事項</strong>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_underscore_3() {
    let input = "正體字。︁_Traditional._";
    let expected = "正體字。︁<em>Traditional.</em>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_underscore_4() {
    let input = "正體字。︁__Hong Kong and Taiwan.__";
    let expected = "正體字。︁<strong>Hong Kong and Taiwan.</strong>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_underscore_5() {
    let input = "简体字 / 新字体。︀_Simplified._";
    let expected = "简体字 / 新字体。︀<em>Simplified.</em>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_underscore_6() {
    let input = "简体字 / 新字体。︀__Mainland China or Japan.__";
    let expected = "简体字 / 新字体。︀<strong>Mainland China or Japan.</strong>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn cjk_underscore_7() {
    let input = "“︁Git”︁__Hub__";
    let expected = "“︁Git”︁<strong>Hub</strong>";

    assert_eq!(expected, harness::parse_inline(input));
}
