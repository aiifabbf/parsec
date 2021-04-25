=======
parsec
=======

.. default-role:: math

Build complicated parsers by composing small parsers

Quick start
===========

Build a parser that extracts as many digits as possible

.. code-block:: rust

    let digit = satisfy(|c| c.is_digit(10));
    let parser = digit.many();
    assert_eq!(dbg!(parser.parse("1234abc")), Some(("1234".to_owned(), "abc")));

Build a parser that matches valid parentheses like ``(()())((()))``

.. code-block:: rust

    fn s(input: &str) -> Option<((), &str)> {
        eof
            .choice(
                string("()")
                    .map(|_| ())
                    .choice(s.between(char('('), char(')'))) // Yes, recursive grammar is possible.
                    .many()
                    .map(|_| ()),
            )
            .parse(input)
    }

    let parser = s; // fn(&str) -> Option<(T, &str)> implements Parser<T>
    assert_eq!(
        dbg!(parser.parse("((()))(()(()))".repeat(100000).as_str())),
        Some(((), ""))
    ); // Don't worry, performance is good.