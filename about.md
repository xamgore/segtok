2015-01-12 segtok

**tl;dr** Surprisingly, it is hard to find a good command-line tool for sentence segmentation and word tokenization that
works well with European languages. Here, I present [segtok](https://pypi.python.org/pypi/segtok), a
**Python 2.7 and 3** package, API, and Unix command-line tool to remedy this shortcoming.

# Text processing pipelines

This is the second in a series of posts that will present lean and elegant text processing tools to take you across the
void from your document collection to the linguistic processing tools (_and back_, but more on that in the future). In
the [last post](https://fnl.es/a-review-of-sparse-sequence-taggers.html), I discussed sequence taggers, as they form the
entry point to natural language processing (NLP). Today I am presenting existing tools and a little library of mine for
pre-processing of Germanic and Romance language text (essentially, because I am not knowledgeable of any others...).
Text processing pipelines roughly consist of the following three steps:

1. Document extraction
2. Text pre-processing
3. Language processing

There are many good tools around for document extraction, in particular Apache [Tika](http://tika.apache.org/) is a
great software library I highly recommend for this task. (As a matter of fact, if you want to improve your programming
skills and see some real-life, clean implementations of nearly all Java/GoF software patterns, have a look at the
innards of Tika.) If you want to extract data from PDFs in particular, you should be looking for your preferred tool in
the huge collection of software that is based on the excellent [xpdf](http://www.foolabs.com/xpdf/) library or even OCR
libraries like [Tesseract](https://code.google.com/p/tesseract-ocr/). As for language processing, there are several
tools for chunking and tagging with dynamic graphical models that you can choose from, as outlined in
an [earlier post](https://fnl.es/a-review-of-sparse-sequence-taggers.html) of mine, and for uncovering more involved
semantic relationships, dependency parsers like [RedShift](https://github.com/syllog1sm/redshift) are available.

# Overview and motivation

However, for the intermediate pre-processing, short from cooking up your own solution, currently the best solution is to
use one of the large natural language processing (NLP) frameworks. There are a few sentence segmentation and
tokenization libraries around, particularly in Perl, but they do not have the desired properties to handle more complex
cases or are long [forgotten](http://mailman.uib.no/public/corpora/2007-October/005429.html). The only more recent,
statistics-based tool I could find is [splitta](https://code.google.com/p/splitta/), but development seems to have died
off yet again. If you check out its Issues page, nobody seems to be fixing the problems, it does not work with Python 3,
and its command-line implementation is not really ready for text processing with Unix. Another example
is [Pragmatic Segmenter](https://www.tm-town.com/natural-language-processing), a rule-based segmenter written in Ruby.
But if you feed it examples with abbreviations or other occurrences of problematic issues I will discuss in this post,
you will see that it performs worse than even the statistical approaches provided by the frameworks discussed next.

This leaves you with [NLTK](http://www.nltk.org)'s 
[PunktTokenizer](http://www.nltk.org/_modules/nltk/tokenize/punkt.html), 
[LingPipe](http://alias-i.com/lingpipe/)'s
[SentenceModel](http://alias-i.com/lingpipe/demos/tutorial/sentences/read-me.html),
[OpenNLP](http://opennlp.apache.org/)'s 
[SentenceDetectorME](http://opennlp.sourceforge.net/api/opennlp/tools/sentdetect/SentenceDetectorME.html), and quite a
few more frameworks that have APIs to bridge that gap. (Again, if you enjoy looking at Java [Kingdom of Nouns...] source
code, check out LingPipe - while commercial, it is really well designed.) However, such a heavy-handed approach is to
break a butterfly with a wheel, and in my personal opinion/experience, none of them are doing a particularly good job at
segmenting, either. If you don't believe me (never do!), just compare their performance/results against the rule-based
library I am about to present here - you will see that these statistical segmenters produce a significant number of
false positives on orthographically (mostly) correct texts. Their strength compared to this library here, however, is
language independence: the library discussed here (so far?) only works with Indo-European languages, while the mentioned
segmenters from the above frameworks can be trained on nearly any language and domain. Particularly, the unsupervised
PunktTokenizer only needs sufficient text and the presence of a terminal marker to learn the segmentation rules on its
own. Similarly, if you want to parse noisy text with bad spelling (Twitter and other "social" media sources), you might
be best advised to use those frameworks. So while I do think these libraries are all great as a whole - and I would
recommend any one of them - it is mildly annoying that you have to learn a framework if all you want to do is common
text pre-processing.

If you are analyzing corporate documents, patents, news articles, scientific texts, technical manuals, web pages, etc.,
that tend to have good orthography these statistical tools are not quite up to it and introduce far too many splits, at
least for my taste. This then affects your downstream language processing, because the errors made by the pre-processing
will be propagated and, in the worst case, even amplified. Therefore, text processing pipelines commonly end up doing
both (2) and (3) using one such framework, but years of experience have shown me that you soon will be wanting to
explore methods beyond whatever particular framework you chose offers. That then can mean that you have to
re-conceptualize all of (2) to add a different tool or even need to move to a whole new framework. If the performance of
the newly integrated framework then isn't that stellar either, it becomes disputable if it even was worth the effort.
Last but not least, this framework-based software development approach violates one of the most fundamental Unix
philosophies:

> "A program should do one thing and it should do it well. Programs should handle _text streams_, because that is a
> universal interface."  
> (Doug McIlroy)

(Yes, I know, it seems this library is violating the "one thing" rule because it does two things: segmenting and
tokenization. But as you will see, the library comes with two independent scripts and APIs for each step.)

The next two sections are for newcomers to this topic and explains why segmenting and tokenizing isn't that trivial as
it might appear. If you are an expert, you can skip the next two sections and read on
where [segtok](https://pypi.python.org/pypi/segtok) is introduced.

# What is sentence segmentation and tokenization?

Nearly any text mining and/or linguistic analysis starts with a sentence and word segmentation step. That is,
determining all the individual spans in a piece of text that constitute its sentences or words. Identifying sentences is
important because they form logical units of thought and represent the borders of many grammatical effects. All our
communication - statements, questions, and commands - are expressed by sentences or at least a meaningful sentence-like
fragment, i.e., a phrase. Words on the other hand are the atomic units that form the sentences, and ultimately, our
language. While words are built up from a sequence of symbols in most languages, these symbols have no semantics of
their own (except for any single-symbol words in that language, naturally). Therefore, nearly any meaningful text
processing task will require the segmentation of the sequence of symbols (characters in computer lingo) into sentences
and words. These words are, at least in the context of computational text processing, often called **tokens**. Beyond
the actual words consisting of letters, a token includes atomic units consisting of other symbols. For example, the
sentence terminals (., !, and ? are three such tokens), a currency symbol, or even chains of symbols (for example, the
ellipsis: ...). By following through with this terminology, the process of segmenting text into these atomic units is
commonly called _tokenization_ and a computer subroutine doing this segmentation is known as a _tokenizer_.

# Sentence segmentation and tokenization is hard

While this segmentation step might initially sound like a rather trivial problem, it turns out the rabbit hole is deep
and no perfect solution has been found to date. Furthermore, the problem is made harder by the different symbols and
their usage in distinct languages. For example, just finding word boundaries in Chinese is non-trivial, because there is
not boundary marker (unlike the whitespace used by Indo-European languages). And when looking into technical documents,
the problem can grow even more out of hand. Names of chemical compounds, web addresses, mathematical expressions, etc.
are all complicating the way one would normally define a set of word boundary detection rules. Another source of
problems are texts from public internet fora, such as Twitter. Words are not spelled as expected, the spaces might be
missing, and emoticons and other ASCII-art can make defining the correct tokenization strategy a rather difficult
endeavor. Similarly, the sentence terminal marker in many Indo-European languages is the full-stop dot -- which
coincidentally is also used as the abbreviation marker in those languages. For example, detecting the (right!) three
sentences in in the following text fragment is not that trivial, at least for a computer program.

> Hello, Mr. Man. He smiled!! This, i.e. that, is it.

If the text fragments are large or the span contains many dots, even humans will start to make many errors when trying
to identify all the sentence boundaries. Certain proper nouns (gene names, or "amnesty international", for example)
might demand that the sentence begins with a lower case letter instead of the expected upper-case. A simple typo might
have been the cause for a sentence starting with a lower-case letter, too. Again in public internet fora, users
sometimes resort to using only lower- or upper-case for their messages or write in an orthographically invalid mix of
letter casing.

# Introducing segtok -- a Python library for these two issues

```sh
pip3 install segtok
```

The Unix approach to software (one thing only, with a text-based interface [that can be used with pipes]) allows you to
integrate programs at each stage of your tool-chain and makes it simple to quickly exchange any parts. If you use a
large framework, on the other hand, you are constrained by it and might feel tempted to accept that some of the things
you will be doing with it aren't done quite as efficiently as possible. And the more you make use of that large
framework, the less attractive it is to switch your tooling and move out of that "comfort zone". I think this issue has
a direct, detrimental effect on our ability to experiment and adapt to new tools, software, and methods.

Due to the many different ways this problem can be solved and the inherent complexity if considering all
languages, [segtok](https://pypi.python.org/pypi/segtok), the library presented here, is confined to processing
orthographically regular Germanic (e.g., English) and Romance (e.g., Spanish) texts. It has a strong focus on those two
and German, which all use Latin letters and standard symbols (like .?!"'([{, etc.). This is mostly based on the fact
that I only know those three languages to some reasonable degree (my tourist-Italian does not count...) - while
help/contributions to make segtok work in more languages would be very welcome!
Furthermore, [segtok](https://pypi.python.org/pypi/segtok) was made to cope with text having the following properties:

- Capable of (correctly!) handling the whole Unicode space.
- A sentence termination marker must be present.
- Texts that follow a mostly regular writing style - in particular, segtok is not tuned for Twitter's highly particular
  orthography.
- It can handle technical texts (containing, e.g., chemical compounds) and internet URIs (IP addresses, URLs and e-mail
  addresses).
- The tool is able to handle (valid) cases of sentences starting with a lower-case letter and correctly splits sentences
  enclosed by parenthesis and/or quotation marks.
- It is able to handle some of the more common cases of heavy abbreviation use (e.g., academic citations).
- It treats all _Unicode_ dashes (there are quite a few of them in Unicode land) "The Right Way" - a functionality
  surprisingly absent from most tools.

Overall, the two scripts that come with segtok have a very simple plain-text, line-based interfaces that work well when
joined with Unix pipe streams. The first script, `segmenter`, segments sentences in (plain) text files into one sentence
per line. The other, `tokenizer`, splits tokens on single lines (usually, the sentences from the `segmenter`) by adding
whitespaces where necessary. On the other hand, if you are a Python developer, you can use the functions ("Look Ma, no
nouns!"...) provided by this library to incorporate this approach in your own software (the tool is MIT licensed, btw.).
Segtok is designed to handle texts with characters from the entire Unicode space, not just ASCII or Latin-1 (ISO-8859-1).

# Sentence segmentation with segtok

```python
from segtok.segmenter import split_single, split_multi
```

On the sentence level, segtok can detect sentences that are contained inside brackets or quotation marks and maintains
those brackets as part of the sentence; For example:

- (A sentence in parenthesis!)
- Or a sentence with "a quote!"
- 'How about handling single quotes?'
- \[Square brackets are fine, too.]

The segmenter is constrained to only segment on single lines ( `split_single` - sentences are not allowed to cross line
boundaries) or to consecutive lines ( `split_multi` - splitting is allowed across newlines inside "paragraphs" separated
by two or more newlines). (If you really want to extract sentences that cross consecutive newline characters, please
remove those line-breaks from your text first. Segtok assumes your content has some minimal semantical meaning, while
superfluous newlines are nothing more than noise.) It gracefully handles enumerations, dots, multiple terminals,
ellipsis, and similar issues:

- A. The first assumption.
- 2. And that is two.
- (iii) Third things here.
- What the heck??!?!
- A terminal ellipsis...

In essence, a valid sentence terminal must be represented by one of the allowed Unicode markers. That is, the many
Unicode variants of ., ?, and !, and the ideographic full-stop: "。" (a single character). Therefore, _this library
cannot guess a sentence boundary if the marker is absent_! After the marker, up to one quotation mark and one bracket
may be present. Finally, the marker must be separated from the following non-space symbol by at least one whitespace
character or a newline.

This requires that the sentence boundaries do obey some limited amount of regularity. But at the same time, the pesky
requirement that a marker is followed by upper-case letters is absent from this strategy. In addition, this means that "
inner" abbreviation markers are never a candidate (such as in "U.S.A."). On the other hand, any markers that do not
follow this "minimal" pattern will always result in false negatives (i.e., not be split). While missing markers and
markers not followed by a space character do occur, those cases are very infrequent in orthographically correct texts.

After these _potential_ markers have been established, the method goes back and looks at the surrounding text to
determine if that marker is not at a sentence boundary after all. This step recuperates cases like initials ("A. Name"),
species names ("S. pombe") and abbreviations inside brackets, which are common with
citations ("[A. Name, B. Other. Title of the work. Proc. Natl. Acad. Sci. 2010]"). Obvious and common abbreviations (in
English, Spanish, and German, so far) followed by a marker are dropped, too. There are several other enhancements to the
segmenter (e.g., checking for the presence of lower-case words that are unlikely start a sentence) that can be studied
in the source code and unit tests. In summary, while coming at a computational cost, this second check is what allows
segtok to keep the number of false positive splits to an acceptable low if compared to existing methods.

# Segtok's tokenization strategy

```python
from segtok.tokenizer import symbol_tokenizer, word_tokenizer, web_tokenizer
from segtok.tokenizer import split_possessive_markers, split_contractions
```

The tokenization approach uses a similar approach. First, a maximal token split is made, and then several functions wrap
this basic approach, encapsulating successively more complex rules that join neighboring tokens back together based on
their orthographic properties. The basic, maximum split rule is to segment everything that is separated by spaces and
then within the remaining non-space spans, split anything that is alphanumeric from any other symbols:

`a123, an alpha-/-beta...` → `a123 , an alpha -/- beta ...`

This functionality is provided by the `symbol_tokenizer`. Next, the non-alphanumeric _symbols_ are further analyzed to
determine if they should form part of a neighboring alphanumeric _word_. If so, the symbols are merged back together
with their alphanumeric spans.

- Abbreviation markers are attached back on to the proceeding word ("Mr.").
- Words with internal dots, dashes, apostrophes, and commas are joined together again ("192.168.1.0", "Abel-Ryan's", 
  "a,b-symmetry").
- The spaces inside a word-hyphen-spaces-word sequence are dropped.
- Superscript and subscript digits, optionally affixed with plus or minus, are attached to a proceeding word that is
  likely to be a physical unit ("m³") or part of a chemical formula, respectively ("[Al₂(S₁O₄)₃]
  ²⁻" → "[", "Al₂", "(", "S₁O₄", ")₃", "]²⁻").

This set of functionality is provided by the `word_tokenizer`. Finally, if desired, a Web-mode function will further
ensure that valid e-mail addresses and URLs (including fragments and parameters, but without actual space characters)
are always maintained as single tokens (`web_tokenizer`). All this ensures that while a decent amount of splitting is
made, the common over-splitting of tokens is avoided. Particularly, when processing biomedical documents, Web content,
or patents, too much tokenization might have quite a significant negative impact on any subsequent, more advanced
processing techniques. As before with the segmenter, I believe this recovery of false positives is the particular
strength of this library.

After the tokenization step, the API provides two functions to optionally split off English possessive markers 
("Fred's", "Argus'") and even contractions ("isn't" → "is n't" \[note the attachment of the letter n], 
"he'll" → "he 'll", "I've" → "I 've", etc.) as their own tokens, which can be useful for downstream linguistic parsing 
(`split_possessive_markers` and `split_contractions`). To use them, just wrap your tokenizer with the preferred method:

```python
def SimpleTokenizer(text):
    for sentence in split_multi(text):
        for token in split_contractions(word_tokenizer(sentence)):
            yield token
        yield None  # None to signal sentence terminals


# An even shorter usage example:
my_tokens = [split_contractions(word_tokenizer(sentence)) for
             sentence in split_multi(my_text)]
```

# Feedback and Conclusions

All this functionality and the API itself are briefly documented on [segtok](https://pypi.python.org/pypi/segtok)'s 
"homepage". As there is not very much functionality around, I hope that between this guide here and the overview there,
the library should be fairly easy to use. Furthermore, in command-line mode, using the `--help` option will explain you
all options provided by the two scripts the PyPI package installations.

If you are be looking for new features, you are welcome to extend the library or request a new feature on the tool's
GitHub [Issues](https://github.com/fnl/segtok/issues) page (no guarantees, though... ;-)). As a forum for discussing
this tool, please use
this [Reddit](http://www.reddit.com/r/Python/comments/2sala9/segtok_a_rulebased_sentence_segmenter_and_word/) thread. In
addition, if you use this library and run into any problems, I would be glad to receive bug reports there, too. Overall,
I have attempted to keep the strategy used by segtok as slim as possible. So if you are using any heavy language
processing or sequence analysis tools after segtok, it should have no impact on your throughput at all.

I have created this library after being disappointed by the other approaches in the wild, and for regular texts, my
experience is that it works substantially superior in at least one of segmentation capabilities and/or runtime
performance. As I do not wish to bash any existing tool, I will only name one sentence segmentation approach I like very
much: Punkt Tokenizer by Kiss and Strunk, 2006. PT is an unsupervised, statistical approach to segmentation that 
"learns" whether to split sentences at sentence terminal markers. While quite impressive and very versatile due to its
unsupervised nature, I can state clearly that segtok's segmenter works substantially better on Germanic and Romance
texts that (mostly) have a proper orthography. Unsurprisingly, segtok's sentence segmenter is substantially faster than
a comparable Python [implementation](http://www.nltk.org/_modules/nltk/tokenize/punkt.html) of the Punkt Tokenizer by
NLTK.
