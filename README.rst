beamdpr
=======

Combine and transform egsphsp (EGS phase space) files.

.. image:: https://travis-ci.org/henrybaxter/beamdpr.svg?branch=master
    :target: https://travis-ci.org/henrybaxter/beamdpr

.. image:: https://img.shields.io/crates/v/beamdpr.svg
    :target: https://crates.io/crates/beamdpr


How it works
~~~~~~~~~~~~

.. code-block:: bash

    $ beamdpr translate -i first.egsphsp -x 2.3
    Done :)

Now ``first.egsphsp`` has been translated 2.3 cm in the positive x direction.


Installation
============

Homebrew:
.. code-block:: bash

    brew tap henrybaxter/tap
    brew install beamdpr

Anything else:

1. Install rust for your system

    https://www.rust-lang.org/en-US/downloads.html

2. Install ``beamdpr`` using ``cargo``

    ``cargo install beamdpr``

3. All done. ``beamdpr`` should work. If not, `please file an issue! <https://github.com/henrybaxter/beamdpr/issues/new>`_


Usage
=====

Start by typing ``beamdpr`` at the terminal/command line. If that does not work, go back to the installation step. Still no dice? `Please file an issue! <https://github.com/henrybaxter/beamdpr/issues/new>`_

Combine
-------

Assume these are your files:

.. code-block:: bash

    $ ls
    first.egsphsp second.egsphsp

Make a combined version:

.. code-block:: bash

    $ beamdpr combine *.egsphsp -o combined.egsphsp
    Done :)

And there it is:

.. code-block:: bash

    $ ls
    combined.egsphsp first.egsphsp second.egsphsp

**NOTE:** This will work with any number of files, as long as their mode matches. That means either they all have ZLAST or none of them do.


Translate
---------

Let's assume this is your file:

.. code-block:: bash

    $ ls
    first.egsphsp

Now translate it 23 in the x direction and -5.7 in the y:

.. code-block:: bash

    $ beamdpr translate first.egsphsp translated.egsphsp -x 23 -y (-5.7)
    Done :)

And there you have it :

.. code-block:: bash

    $ ls
    first.egsphsp translated.egsphsp

**NOTE:** Negative numbers must have parantheses around them. You may omit an argument if you only want to translate in one direction.

Rotate
------

Let's assume this is your file:

.. code-block:: bash

    $ ls first.egsphsp
    first.egsphsp

Now rotate .9 radians in the counter-clockwise direction:

.. code-block:: bash

    $ beamdpr rotate first.egsphsp rotated.egsphsp --angle .9
    Done :)

It's all done!

.. code-block:: bash

    $ ls
    first.egsphsp rotated.egsphsp


**NOTE:** If you rotate by 2π (6.28318530718) the file should be unchanged right? Not quite. Due to floating point vagaries there will be minor binary differences, but the value differences will be negligible.


Reflect
-------

Let's assume this is your file:

.. code-block:: bash

    $ ls first.egsphsp
    first.egsphsp

Now reflect around the vector (1, 0):

.. code-block:: bash

    $ beamdpr rotate first.egsphsp reflected.egsphsp -x 1
    Done :)

All set!

.. code-block:: bash

    $ ls
    first.egsphsp reflected.egsphsp

**NOTE:** This effectively changed the sign of all y values and y directions.


In-place
--------

Any of these transform operations can be done **in-place** - that is, by modifying the input file, rather than creating a new one:

.. code-block:: bash

    $ ls first.egsphsp
    first.egsphsp

Reflect in the vector (-1, 0) in-place:

.. code-block:: bash

    $ beamdpr rotate first.egsphsp -i -x (-1)
    Done :)

**NOTE:** Negative numbers are input using parantheses, and ``-i`` is the same as ``--in-place``.

Delete after reading
--------------------

During a combine operation you may worry about disk space (10x10gb of phase space files could add another 100gb of combined phase space files). Let's delete as we go:

.. code-block:: bash

    $ ls
    first.egsphsp second.egsphsp

So make a combined version:

.. code-block:: bash

    $ beamdpr combine *.egsphsp -o combined.egsphsp -d
    Done :)

.. code-block:: bash

    $ ls
    combined.egsphsp


Help
====

Stuck? `Please file an issue! <https://github.com/henrybaxter/beamdpr/issues/new>`_
