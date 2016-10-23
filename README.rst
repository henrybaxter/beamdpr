beamdpr
=======

Combine and transform egsphsp (EGS phase space) files.


Installation
============

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

Let's assume these are your files:

.. code-block:: bash

    $ ls
    first.egsphsp second.egsphsp

So make a combined version:

.. code-block:: bash

    $ beamdpr combine *.egsphsp -o combined.egsphsp
    Done :)

Sure enough if it's there:

.. code-block:: bash

    $ ls
    combined.egsphsp first.egsphsp second.egsphsp

Translate
---------

Let's assume this is your file:

.. code-block:: bash

    $ ls first.egsphsp
    first.egsphsp

Now translate it 23 in the x direction and -5.7 in the y:

.. code-block:: bash

    $ beamdpr translate first.egsphsp -x 23 -y -5.7
    Done :)

And there you have it :

.. code-block: bash

    $ ls
    first.egsphsp translated.egsphsp

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

.. code-block: bash

    $ ls
    first.egsphsp rotated.egsphsp


Reflect
-------
Let's assume this is your file:

.. code-block:: bash

    $ ls first.egsphsp
    first.egsphsp

Now reflect in the vector (1, 0):

.. code-block:: bash

    $ beamdpr rotate first.egsphsp reflected.egsphsp -x 1
    Done :)

All set!

.. code-block: bash

    $ ls
    first.egsphsp reflected.egsphsp


In-place
--------

Any of these transform operations can be done **in-place** - that is, by modifying the input file, rather than creating a new one:

.. code-block:: bash

    $ ls first.egsphsp
    first.egsphsp

Reflect in the vector (-1, 0) in-place:

.. code-block:: bash

    $ beamdpr rotate first.egsphsp -i -x 1
    Done :)

Note that ``-i`` is the same as ``--in-place``.

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
