macro_rules! corpus_test {
    ($name:ident, $path:literal, $snapshot:literal) => {
        #[test]
        fn $name() {
            let input = include_str!($path);
            let result = etch_core::parse(input);
            insta::assert_json_snapshot!($snapshot, result);
        }
    };
}

// Corpus inventory:
// - core/blockquotes/basic.etch
// - core/blockquotes/multi-paragraph.etch
// - core/blockquotes/nested.etch
// - core/blockquotes/with-attribution.etch
// - core/blockquotes/with-formatting.etch
// - core/code-blocks/backticks-inside.etch
// - core/code-blocks/basic.etch
// - core/code-blocks/empty.etch
// - core/code-blocks/with-etch-inside.etch
// - core/code-blocks/with-language.etch
// - core/definition-lists/basic.etch
// - core/definition-lists/multiple-defs.etch
// - core/definition-lists/multiple-terms.etch
// - core/definition-lists/with-formatting.etch
// - core/footnotes/basic.etch
// - core/footnotes/multi-paragraph.etch
// - core/footnotes/multiple.etch
// - core/footnotes/named.etch
// - core/hard-line-breaks/backslash.etch
// - core/hard-line-breaks/in-blockquote.etch
// - core/hard-line-breaks/in-list.etch
// - core/hard-line-breaks/two-spaces-no-break.etch
// - core/headings/all-levels.etch
// - core/headings/followed-by-paragraph.etch
// - core/headings/not-a-heading.etch
// - core/headings/seven-hashes.etch
// - core/headings/with-inline.etch
// - core/inline/across-lines.etch
// - core/inline/adjacent.etch
// - core/inline/emphasis.etch
// - core/inline/empty-markers.etch
// - core/inline/escaped.etch
// - core/inline/highlight.etch
// - core/inline/inline-code.etch
// - core/inline/insert.etch
// - core/inline/nested.etch
// - core/inline/strikethrough.etch
// - core/inline/strong.etch
// - core/inline/strong-emphasis.etch
// - core/inline/subscript.etch
// - core/inline/superscript.etch
// - core/links-images/autolink.etch
// - core/links-images/basic-image.etch
// - core/links-images/basic-link.etch
// - core/links-images/image-with-attrs.etch
// - core/links-images/image-with-title.etch
// - core/links-images/link-with-formatting.etch
// - core/links-images/link-with-title.etch
// - core/lists/mixed.etch
// - core/lists/nested.etch
// - core/lists/ordered.etch
// - core/lists/task-list.etch
// - core/lists/unordered.etch
// - core/lists/with-formatting.etch
// - core/lists/with-paragraphs.etch
// - core/paragraphs/empty.etch
// - core/paragraphs/extra-blanks.etch
// - core/paragraphs/multi-line.etch
// - core/paragraphs/multiple.etch
// - core/paragraphs/single.etch
// - core/paragraphs/trailing-whitespace.etch
// - core/paragraphs/whitespace-only.etch
// - core/tables/alignment.etch
// - core/tables/basic.etch
// - core/tables/many-columns.etch
// - core/tables/single-column.etch
// - core/tables/with-formatting.etch
// - core/thematic-breaks/all-variants.etch
// - core/thematic-breaks/basic.etch
// - core/thematic-breaks/long-variants.etch
// - core/thematic-breaks/not-frontmatter.etch
// - extensions/attributes/combined.etch
// - extensions/attributes/escaped-quote.etch
// - extensions/attributes/on-blockquote.etch
// - extensions/attributes/on-code-block.etch
// - extensions/attributes/on-heading.etch
// - extensions/attributes/on-image.etch
// - extensions/attributes/on-paragraph.etch
// - extensions/attributes/on-table.etch
// - extensions/block-directives/basic.etch
// - extensions/block-directives/blank-lines-inside.etch
// - extensions/block-directives/empty-body.etch
// - extensions/block-directives/math.etch
// - extensions/block-directives/multiple.etch
// - extensions/block-directives/rich-content.etch
// - extensions/block-directives/with-attrs.etch
// - extensions/block-directives/with-both.etch
// - extensions/block-directives/with-label.etch
// - extensions/comments/adjacent-to-attrs.etch
// - extensions/comments/in-heading.etch
// - extensions/comments/inline.etch
// - extensions/comments/line.etch
// - extensions/comments/multi-line.etch
// - extensions/comments/nested-tilde.etch
// - extensions/comments/no-nesting.etch
// - extensions/container-directives/anonymous-close.etch
// - extensions/container-directives/columns-pattern.etch
// - extensions/container-directives/mismatched-close.etch
// - extensions/container-directives/named-close.etch
// - extensions/container-directives/nested-containers.etch
// - extensions/container-directives/with-attrs.etch
// - extensions/container-directives/with-block-directives.etch
// - extensions/container-directives/with-blocks-inside.etch
// - extensions/frontmatter/basic.etch
// - extensions/frontmatter/empty.etch
// - extensions/frontmatter/full.etch
// - extensions/frontmatter/no-frontmatter.etch
// - extensions/frontmatter/not-first-line.etch
// - extensions/inline-directives/balanced-brackets.etch
// - extensions/inline-directives/bare.etch
// - extensions/inline-directives/content-with-formatting.etch
// - extensions/inline-directives/escaped-bracket.etch
// - extensions/inline-directives/multiple-in-paragraph.etch
// - extensions/inline-directives/not-a-directive.etch
// - extensions/inline-directives/quoted-attr-values.etch
// - extensions/inline-directives/with-attrs.etch
// - extensions/inline-directives/with-both.etch
// - extensions/inline-directives/with-content.etch
// - extensions/nesting/depth-4-warning.etch
// - extensions/nesting/inline-in-leaf.etch
// - extensions/nesting/leaf-contains-text.etch
// - extensions/nesting/leaf-rejects-directive.etch
// - extensions/nesting/structural-in-structural.etch
// - integration/embers-in-the-snow.etch
// - integration/everything.etch
// - integration/minimal.etch
// - integration/plain-story.etch
// - integration/technical-doc.etch

// === Core: Blockquotes ===
corpus_test!(core_blockquotes_basic, "../../../tests/corpus/core/blockquotes/basic.etch", "core-blockquotes-basic");
corpus_test!(core_blockquotes_multi_paragraph, "../../../tests/corpus/core/blockquotes/multi-paragraph.etch", "core-blockquotes-multi-paragraph");
corpus_test!(core_blockquotes_nested, "../../../tests/corpus/core/blockquotes/nested.etch", "core-blockquotes-nested");
corpus_test!(core_blockquotes_with_attribution, "../../../tests/corpus/core/blockquotes/with-attribution.etch", "core-blockquotes-with-attribution");
corpus_test!(core_blockquotes_with_formatting, "../../../tests/corpus/core/blockquotes/with-formatting.etch", "core-blockquotes-with-formatting");

// === Core: Code Blocks ===
corpus_test!(core_code_blocks_backticks_inside, "../../../tests/corpus/core/code-blocks/backticks-inside.etch", "core-code-blocks-backticks-inside");
corpus_test!(core_code_blocks_basic, "../../../tests/corpus/core/code-blocks/basic.etch", "core-code-blocks-basic");
corpus_test!(core_code_blocks_empty, "../../../tests/corpus/core/code-blocks/empty.etch", "core-code-blocks-empty");
corpus_test!(core_code_blocks_with_etch_inside, "../../../tests/corpus/core/code-blocks/with-etch-inside.etch", "core-code-blocks-with-etch-inside");
corpus_test!(core_code_blocks_with_language, "../../../tests/corpus/core/code-blocks/with-language.etch", "core-code-blocks-with-language");

// === Core: Definition Lists ===
corpus_test!(core_definition_lists_basic, "../../../tests/corpus/core/definition-lists/basic.etch", "core-definition-lists-basic");
corpus_test!(core_definition_lists_multiple_defs, "../../../tests/corpus/core/definition-lists/multiple-defs.etch", "core-definition-lists-multiple-defs");
corpus_test!(core_definition_lists_multiple_terms, "../../../tests/corpus/core/definition-lists/multiple-terms.etch", "core-definition-lists-multiple-terms");
corpus_test!(core_definition_lists_with_formatting, "../../../tests/corpus/core/definition-lists/with-formatting.etch", "core-definition-lists-with-formatting");

// === Core: Footnotes ===
corpus_test!(core_footnotes_basic, "../../../tests/corpus/core/footnotes/basic.etch", "core-footnotes-basic");
corpus_test!(core_footnotes_multi_paragraph, "../../../tests/corpus/core/footnotes/multi-paragraph.etch", "core-footnotes-multi-paragraph");
corpus_test!(core_footnotes_multiple, "../../../tests/corpus/core/footnotes/multiple.etch", "core-footnotes-multiple");
corpus_test!(core_footnotes_named, "../../../tests/corpus/core/footnotes/named.etch", "core-footnotes-named");

// === Core: Hard Line Breaks ===
corpus_test!(core_hard_line_breaks_backslash, "../../../tests/corpus/core/hard-line-breaks/backslash.etch", "core-hard-line-breaks-backslash");
corpus_test!(core_hard_line_breaks_in_blockquote, "../../../tests/corpus/core/hard-line-breaks/in-blockquote.etch", "core-hard-line-breaks-in-blockquote");
corpus_test!(core_hard_line_breaks_in_list, "../../../tests/corpus/core/hard-line-breaks/in-list.etch", "core-hard-line-breaks-in-list");
corpus_test!(core_hard_line_breaks_two_spaces_no_break, "../../../tests/corpus/core/hard-line-breaks/two-spaces-no-break.etch", "core-hard-line-breaks-two-spaces-no-break");

// === Core: Headings ===
corpus_test!(core_headings_all_levels, "../../../tests/corpus/core/headings/all-levels.etch", "core-headings-all-levels");
corpus_test!(core_headings_followed_by_paragraph, "../../../tests/corpus/core/headings/followed-by-paragraph.etch", "core-headings-followed-by-paragraph");
corpus_test!(core_headings_not_a_heading, "../../../tests/corpus/core/headings/not-a-heading.etch", "core-headings-not-a-heading");
corpus_test!(core_headings_seven_hashes, "../../../tests/corpus/core/headings/seven-hashes.etch", "core-headings-seven-hashes");
corpus_test!(core_headings_with_inline, "../../../tests/corpus/core/headings/with-inline.etch", "core-headings-with-inline");

// === Core: Inline ===
corpus_test!(core_inline_across_lines, "../../../tests/corpus/core/inline/across-lines.etch", "core-inline-across-lines");
corpus_test!(core_inline_adjacent, "../../../tests/corpus/core/inline/adjacent.etch", "core-inline-adjacent");
corpus_test!(core_inline_emphasis, "../../../tests/corpus/core/inline/emphasis.etch", "core-inline-emphasis");
corpus_test!(core_inline_empty_markers, "../../../tests/corpus/core/inline/empty-markers.etch", "core-inline-empty-markers");
corpus_test!(core_inline_escaped, "../../../tests/corpus/core/inline/escaped.etch", "core-inline-escaped");
corpus_test!(core_inline_highlight, "../../../tests/corpus/core/inline/highlight.etch", "core-inline-highlight");
corpus_test!(core_inline_inline_code, "../../../tests/corpus/core/inline/inline-code.etch", "core-inline-inline-code");
corpus_test!(core_inline_insert, "../../../tests/corpus/core/inline/insert.etch", "core-inline-insert");
corpus_test!(core_inline_nested, "../../../tests/corpus/core/inline/nested.etch", "core-inline-nested");
corpus_test!(core_inline_strikethrough, "../../../tests/corpus/core/inline/strikethrough.etch", "core-inline-strikethrough");
corpus_test!(core_inline_strong, "../../../tests/corpus/core/inline/strong.etch", "core-inline-strong");
corpus_test!(core_inline_strong_emphasis, "../../../tests/corpus/core/inline/strong-emphasis.etch", "core-inline-strong-emphasis");
corpus_test!(core_inline_subscript, "../../../tests/corpus/core/inline/subscript.etch", "core-inline-subscript");
corpus_test!(core_inline_superscript, "../../../tests/corpus/core/inline/superscript.etch", "core-inline-superscript");

// === Core: Links Images ===
corpus_test!(core_links_images_autolink, "../../../tests/corpus/core/links-images/autolink.etch", "core-links-images-autolink");
corpus_test!(core_links_images_basic_image, "../../../tests/corpus/core/links-images/basic-image.etch", "core-links-images-basic-image");
corpus_test!(core_links_images_basic_link, "../../../tests/corpus/core/links-images/basic-link.etch", "core-links-images-basic-link");
corpus_test!(core_links_images_image_with_attrs, "../../../tests/corpus/core/links-images/image-with-attrs.etch", "core-links-images-image-with-attrs");
corpus_test!(core_links_images_image_with_title, "../../../tests/corpus/core/links-images/image-with-title.etch", "core-links-images-image-with-title");
corpus_test!(core_links_images_link_with_formatting, "../../../tests/corpus/core/links-images/link-with-formatting.etch", "core-links-images-link-with-formatting");
corpus_test!(core_links_images_link_with_title, "../../../tests/corpus/core/links-images/link-with-title.etch", "core-links-images-link-with-title");

// === Core: Lists ===
corpus_test!(core_lists_mixed, "../../../tests/corpus/core/lists/mixed.etch", "core-lists-mixed");
corpus_test!(core_lists_nested, "../../../tests/corpus/core/lists/nested.etch", "core-lists-nested");
corpus_test!(core_lists_ordered, "../../../tests/corpus/core/lists/ordered.etch", "core-lists-ordered");
corpus_test!(core_lists_task_list, "../../../tests/corpus/core/lists/task-list.etch", "core-lists-task-list");
corpus_test!(core_lists_unordered, "../../../tests/corpus/core/lists/unordered.etch", "core-lists-unordered");
corpus_test!(core_lists_with_formatting, "../../../tests/corpus/core/lists/with-formatting.etch", "core-lists-with-formatting");
corpus_test!(core_lists_with_paragraphs, "../../../tests/corpus/core/lists/with-paragraphs.etch", "core-lists-with-paragraphs");

// === Core: Paragraphs ===
corpus_test!(core_paragraphs_empty, "../../../tests/corpus/core/paragraphs/empty.etch", "core-paragraphs-empty");
corpus_test!(core_paragraphs_extra_blanks, "../../../tests/corpus/core/paragraphs/extra-blanks.etch", "core-paragraphs-extra-blanks");
corpus_test!(core_paragraphs_multi_line, "../../../tests/corpus/core/paragraphs/multi-line.etch", "core-paragraphs-multi-line");
corpus_test!(core_paragraphs_multiple, "../../../tests/corpus/core/paragraphs/multiple.etch", "core-paragraphs-multiple");
corpus_test!(core_paragraphs_single, "../../../tests/corpus/core/paragraphs/single.etch", "core-paragraphs-single");
corpus_test!(core_paragraphs_trailing_whitespace, "../../../tests/corpus/core/paragraphs/trailing-whitespace.etch", "core-paragraphs-trailing-whitespace");
corpus_test!(core_paragraphs_whitespace_only, "../../../tests/corpus/core/paragraphs/whitespace-only.etch", "core-paragraphs-whitespace-only");

// === Core: Tables ===
corpus_test!(core_tables_alignment, "../../../tests/corpus/core/tables/alignment.etch", "core-tables-alignment");
corpus_test!(core_tables_basic, "../../../tests/corpus/core/tables/basic.etch", "core-tables-basic");
corpus_test!(core_tables_many_columns, "../../../tests/corpus/core/tables/many-columns.etch", "core-tables-many-columns");
corpus_test!(core_tables_single_column, "../../../tests/corpus/core/tables/single-column.etch", "core-tables-single-column");
corpus_test!(core_tables_with_formatting, "../../../tests/corpus/core/tables/with-formatting.etch", "core-tables-with-formatting");

// === Core: Thematic Breaks ===
corpus_test!(core_thematic_breaks_all_variants, "../../../tests/corpus/core/thematic-breaks/all-variants.etch", "core-thematic-breaks-all-variants");
corpus_test!(core_thematic_breaks_basic, "../../../tests/corpus/core/thematic-breaks/basic.etch", "core-thematic-breaks-basic");
corpus_test!(core_thematic_breaks_long_variants, "../../../tests/corpus/core/thematic-breaks/long-variants.etch", "core-thematic-breaks-long-variants");
corpus_test!(core_thematic_breaks_not_frontmatter, "../../../tests/corpus/core/thematic-breaks/not-frontmatter.etch", "core-thematic-breaks-not-frontmatter");

// === Extensions: Attributes ===
corpus_test!(extensions_attributes_combined, "../../../tests/corpus/extensions/attributes/combined.etch", "extensions-attributes-combined");
corpus_test!(extensions_attributes_escaped_quote, "../../../tests/corpus/extensions/attributes/escaped-quote.etch", "extensions-attributes-escaped-quote");
corpus_test!(extensions_attributes_on_blockquote, "../../../tests/corpus/extensions/attributes/on-blockquote.etch", "extensions-attributes-on-blockquote");
corpus_test!(extensions_attributes_on_code_block, "../../../tests/corpus/extensions/attributes/on-code-block.etch", "extensions-attributes-on-code-block");
corpus_test!(extensions_attributes_on_heading, "../../../tests/corpus/extensions/attributes/on-heading.etch", "extensions-attributes-on-heading");
corpus_test!(extensions_attributes_on_image, "../../../tests/corpus/extensions/attributes/on-image.etch", "extensions-attributes-on-image");
corpus_test!(extensions_attributes_on_paragraph, "../../../tests/corpus/extensions/attributes/on-paragraph.etch", "extensions-attributes-on-paragraph");
corpus_test!(extensions_attributes_on_table, "../../../tests/corpus/extensions/attributes/on-table.etch", "extensions-attributes-on-table");

// === Extensions: Block Directives ===
corpus_test!(extensions_block_directives_basic, "../../../tests/corpus/extensions/block-directives/basic.etch", "extensions-block-directives-basic");
corpus_test!(extensions_block_directives_blank_lines_inside, "../../../tests/corpus/extensions/block-directives/blank-lines-inside.etch", "extensions-block-directives-blank-lines-inside");
corpus_test!(extensions_block_directives_empty_body, "../../../tests/corpus/extensions/block-directives/empty-body.etch", "extensions-block-directives-empty-body");
corpus_test!(extensions_block_directives_math, "../../../tests/corpus/extensions/block-directives/math.etch", "extensions-block-directives-math");
corpus_test!(extensions_block_directives_multiple, "../../../tests/corpus/extensions/block-directives/multiple.etch", "extensions-block-directives-multiple");
corpus_test!(extensions_block_directives_rich_content, "../../../tests/corpus/extensions/block-directives/rich-content.etch", "extensions-block-directives-rich-content");
corpus_test!(extensions_block_directives_with_attrs, "../../../tests/corpus/extensions/block-directives/with-attrs.etch", "extensions-block-directives-with-attrs");
corpus_test!(extensions_block_directives_with_both, "../../../tests/corpus/extensions/block-directives/with-both.etch", "extensions-block-directives-with-both");
corpus_test!(extensions_block_directives_with_label, "../../../tests/corpus/extensions/block-directives/with-label.etch", "extensions-block-directives-with-label");

// === Extensions: Comments ===
corpus_test!(extensions_comments_adjacent_to_attrs, "../../../tests/corpus/extensions/comments/adjacent-to-attrs.etch", "extensions-comments-adjacent-to-attrs");
corpus_test!(extensions_comments_in_heading, "../../../tests/corpus/extensions/comments/in-heading.etch", "extensions-comments-in-heading");
corpus_test!(extensions_comments_inline, "../../../tests/corpus/extensions/comments/inline.etch", "extensions-comments-inline");
corpus_test!(extensions_comments_line, "../../../tests/corpus/extensions/comments/line.etch", "extensions-comments-line");
corpus_test!(extensions_comments_multi_line, "../../../tests/corpus/extensions/comments/multi-line.etch", "extensions-comments-multi-line");
corpus_test!(extensions_comments_nested_tilde, "../../../tests/corpus/extensions/comments/nested-tilde.etch", "extensions-comments-nested-tilde");
corpus_test!(extensions_comments_no_nesting, "../../../tests/corpus/extensions/comments/no-nesting.etch", "extensions-comments-no-nesting");

// === Extensions: Container Directives ===
corpus_test!(extensions_container_directives_anonymous_close, "../../../tests/corpus/extensions/container-directives/anonymous-close.etch", "extensions-container-directives-anonymous-close");
corpus_test!(extensions_container_directives_columns_pattern, "../../../tests/corpus/extensions/container-directives/columns-pattern.etch", "extensions-container-directives-columns-pattern");
corpus_test!(extensions_container_directives_mismatched_close, "../../../tests/corpus/extensions/container-directives/mismatched-close.etch", "extensions-container-directives-mismatched-close");
corpus_test!(extensions_container_directives_named_close, "../../../tests/corpus/extensions/container-directives/named-close.etch", "extensions-container-directives-named-close");
corpus_test!(extensions_container_directives_nested_containers, "../../../tests/corpus/extensions/container-directives/nested-containers.etch", "extensions-container-directives-nested-containers");
corpus_test!(extensions_container_directives_with_attrs, "../../../tests/corpus/extensions/container-directives/with-attrs.etch", "extensions-container-directives-with-attrs");
corpus_test!(extensions_container_directives_with_block_directives, "../../../tests/corpus/extensions/container-directives/with-block-directives.etch", "extensions-container-directives-with-block-directives");
corpus_test!(extensions_container_directives_with_blocks_inside, "../../../tests/corpus/extensions/container-directives/with-blocks-inside.etch", "extensions-container-directives-with-blocks-inside");

// === Extensions: Frontmatter ===
corpus_test!(extensions_frontmatter_basic, "../../../tests/corpus/extensions/frontmatter/basic.etch", "extensions-frontmatter-basic");
corpus_test!(extensions_frontmatter_empty, "../../../tests/corpus/extensions/frontmatter/empty.etch", "extensions-frontmatter-empty");
corpus_test!(extensions_frontmatter_full, "../../../tests/corpus/extensions/frontmatter/full.etch", "extensions-frontmatter-full");
corpus_test!(extensions_frontmatter_no_frontmatter, "../../../tests/corpus/extensions/frontmatter/no-frontmatter.etch", "extensions-frontmatter-no-frontmatter");
corpus_test!(extensions_frontmatter_not_first_line, "../../../tests/corpus/extensions/frontmatter/not-first-line.etch", "extensions-frontmatter-not-first-line");

// === Extensions: Inline Directives ===
corpus_test!(extensions_inline_directives_balanced_brackets, "../../../tests/corpus/extensions/inline-directives/balanced-brackets.etch", "extensions-inline-directives-balanced-brackets");
corpus_test!(extensions_inline_directives_bare, "../../../tests/corpus/extensions/inline-directives/bare.etch", "extensions-inline-directives-bare");
corpus_test!(extensions_inline_directives_content_with_formatting, "../../../tests/corpus/extensions/inline-directives/content-with-formatting.etch", "extensions-inline-directives-content-with-formatting");
corpus_test!(extensions_inline_directives_escaped_bracket, "../../../tests/corpus/extensions/inline-directives/escaped-bracket.etch", "extensions-inline-directives-escaped-bracket");
corpus_test!(extensions_inline_directives_multiple_in_paragraph, "../../../tests/corpus/extensions/inline-directives/multiple-in-paragraph.etch", "extensions-inline-directives-multiple-in-paragraph");
corpus_test!(extensions_inline_directives_not_a_directive, "../../../tests/corpus/extensions/inline-directives/not-a-directive.etch", "extensions-inline-directives-not-a-directive");
corpus_test!(extensions_inline_directives_quoted_attr_values, "../../../tests/corpus/extensions/inline-directives/quoted-attr-values.etch", "extensions-inline-directives-quoted-attr-values");
corpus_test!(extensions_inline_directives_with_attrs, "../../../tests/corpus/extensions/inline-directives/with-attrs.etch", "extensions-inline-directives-with-attrs");
corpus_test!(extensions_inline_directives_with_both, "../../../tests/corpus/extensions/inline-directives/with-both.etch", "extensions-inline-directives-with-both");
corpus_test!(extensions_inline_directives_with_content, "../../../tests/corpus/extensions/inline-directives/with-content.etch", "extensions-inline-directives-with-content");

// === Extensions: Nesting ===
corpus_test!(extensions_nesting_depth_4_warning, "../../../tests/corpus/extensions/nesting/depth-4-warning.etch", "extensions-nesting-depth-4-warning");
corpus_test!(extensions_nesting_inline_in_leaf, "../../../tests/corpus/extensions/nesting/inline-in-leaf.etch", "extensions-nesting-inline-in-leaf");
corpus_test!(extensions_nesting_leaf_contains_text, "../../../tests/corpus/extensions/nesting/leaf-contains-text.etch", "extensions-nesting-leaf-contains-text");
corpus_test!(extensions_nesting_leaf_rejects_directive, "../../../tests/corpus/extensions/nesting/leaf-rejects-directive.etch", "extensions-nesting-leaf-rejects-directive");
corpus_test!(extensions_nesting_structural_in_structural, "../../../tests/corpus/extensions/nesting/structural-in-structural.etch", "extensions-nesting-structural-in-structural");

// === Integration ===
corpus_test!(integration_embers_in_the_snow, "../../../tests/corpus/integration/embers-in-the-snow.etch", "integration-embers-in-the-snow");
corpus_test!(integration_everything, "../../../tests/corpus/integration/everything.etch", "integration-everything");
corpus_test!(integration_minimal, "../../../tests/corpus/integration/minimal.etch", "integration-minimal");
corpus_test!(integration_plain_story, "../../../tests/corpus/integration/plain-story.etch", "integration-plain-story");
corpus_test!(integration_technical_doc, "../../../tests/corpus/integration/technical-doc.etch", "integration-technical-doc");
