mod harness;

mod icu_inline {
    use crate::harness::icu_string_test;
    icu_string_test!(basic_icu, "{username}", "{username}");
    icu_string_test!(icu_whitespace, "{  username\n}", "{username}");
    icu_string_test!(basic_markdown, "**hello**", "<b>hello</b>");
    icu_string_test!(
        nested_markdown,
        "***hello** this has more content*",
        "<i><b>hello</b> this has more content</i>"
    );
    icu_string_test!(
        static_link,
        "[a link](to/somewhere)",
        "<link>to/somewhere{_}a link</link>"
    );
    icu_string_test!(
        dynamic_link,
        "[a link]({variable})",
        "<link>{variable}a link</link>"
    );
    icu_string_test!(
        autolink,
        "<https://example.com>",
        "<link>https://example.com{_}https://example.com</link>"
    );
    icu_string_test!(no_block_atx_heading, "# false heading", "# false heading");
    icu_string_test!(
        no_block_setext_heading,
        "false setext\n---",
        "false setext\n---"
    );
}

mod icu_variable_formats {
    use crate::harness::icu_string_test;

    icu_string_test!(
        date_word_format,
        "{today, date, short}",
        "{today, date, short}"
    );
    icu_string_test!(
        number_currency_format,
        "{count, number, currency/USD}",
        "{count, number, currency/USD}"
    );
}

mod icu_markdown_blocks {
    use crate::harness::icu_block_string_test;

    icu_block_string_test!(
        atx_heading,
        "# Heading\nwith a paragraph",
        "<h1>Heading</h1>\n<p>with a paragraph</p>"
    );
    icu_block_string_test!(
        setext_heading,
        "Setext *Heading*\n---",
        "<h2>Setext <i>Heading</i></h2>"
    );
    icu_block_string_test!(
        indented_code_block_ignores,
        "    {\n    novariable\n}",
        "<codeBlock>{\nnovariable\n</codeBlock>\n<p>}</p>"
    );
    icu_block_string_test!(
        fenced_code_block_info_string_ignores,
        "``` {novariable}\n```",
        "<codeBlock></codeBlock>"
    );
    icu_block_string_test!(
        fenced_code_block_content_ignores,
        "```\n{\nnovariable\n}\n```",
        "<codeBlock>{\nnovariable\n}\n</codeBlock>"
    );
}

mod icu_blocks {
    use crate::harness::icu_block_string_test;

    icu_block_string_test!(
        beginning_of_paragraph,
        "{variable}\nlazy paragraph continuation",
        "<p>{variable}\nlazy paragraph continuation</p>"
    );
    icu_block_string_test!(
        end_of_paragraph,
        "paragraph start\n{variable}",
        "<p>paragraph start\n{variable}</p>"
    );
    icu_block_string_test!(
        blank_line_start,
        "paragraph one\n\n{variable} paragraph two",
        "<p>paragraph one</p>\n<p>{variable} paragraph two</p>"
    );
    icu_block_string_test!(
        blank_line_end,
        "{variable}\n\nparagraph two",
        "<p>{variable}</p>\n<p>paragraph two</p>"
    );
    icu_block_string_test!(
        continuation_blank_line_inside_var,
        "paragraph one {\n\nvariable\n\n} still paragraph one",
        "<p>paragraph one {variable} still paragraph one</p>"
    );
    icu_block_string_test!(continuation_blank_line_inside_plural_control, "paragraph one {count,\n\n    plural,\n\n one {same par} other\n\n{same paragraph}} still paragraph one","<p>paragraph one {count, plural, one {same par} other {same paragraph}} still paragraph one</p>");
    icu_block_string_test!(continuation_blank_line_inside_plural_value, "paragraph one {count, plural, one {\n\n# false heading\n\n ends with paragraph}} still paragraph one","<p>paragraph one {count, plural, one {# false heading\nends with paragraph}} still paragraph one</p>");
    icu_block_string_test!(
        continuation_blank_line_inside_select_value,
        "this cat is {color, select, orange {orange flavored\n\n} black {void}}",
        "<p>this cat is {color, select, orange {orange flavored} black {void}}</p>"
    );
}

mod icu_in_headings {
    use crate::harness::icu_block_string_test;

    icu_block_string_test!(
        same_line_icu_variable,
        "# Heading {variable}",
        "<h1>Heading {variable}</h1>"
    );
    icu_block_string_test!(
        multiline_icu_variable,
        "# Heading {\n\nvariable}",
        "<h1>Heading {variable}</h1>"
    );
    icu_block_string_test!(
        content_after_icu,
        "# {\n\nvariable} and more",
        "<h1>{variable} and more</h1>"
    );
    icu_block_string_test!(
        setext_heading,
        "{\n\nvariable}\nsetext heading\n===",
        "<h1>{variable}\nsetext heading</h1>"
    );
}

mod icu_escapes {
    use crate::harness::icu_string_test;

    icu_string_test!(icu_escapes, "'{  variable  }", "'{  variable  }");
}
