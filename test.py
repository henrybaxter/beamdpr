from subprocess import run, PIPE
import filecmp
from unittest import TestCase, main
import os
import math


class BinTestCase(TestCase):
    BASE_DIR = 'test_data'

    def test_no_subcommand(self):
        result = run(['cargo', 'run', '--', 'rotate'], stderr=PIPE, stdout=PIPE)
        self.assertEqual(result.returncode, 1)

    def test_combine_no_args(self):
        result = run(['cargo', 'run', '--', 'rotate', 'combine'], stderr=PIPE, stdout=PIPE)
        self.assertEqual(result.returncode, 1)

    def test_combine_one(self):
        input_files = ['test_data/first.egsphsp1']
        output_file = 'test_data/test_combined_output.egsphsp1'
        result = run(['cargo', 'run', '--', 'combine'] + input_files + ['-o', output_file], stderr=PIPE, stdout=PIPE)
        self.assertEqual(result.returncode, 0)
        self.assertTrue(filecmp.cmp(output_file, 'test_data/first.egsphsp1', shallow=False))

    def test_combine_two(self):
        input_files = ['test_data/first.egsphsp1', 'test_data/second.egsphsp1']
        output_file = 'test_data/test_combined_output.egsphsp1'
        result = run(['cargo', 'run', '--', 'combine'] + input_files + ['-o', output_file], stderr=PIPE, stdout=PIPE)
        self.assertEqual(result.returncode, 0)
        self.assertTrue(filecmp.cmp(output_file, 'test_data/combined.egsphsp1', shallow=False))
        os.remove(output_file)

    def test_translate(self):
        input_file = 'test_data/first.egsphsp1'
        output_file = 'test_data/test_translate.egsphsp1'
        result = run(['cargo', 'run', '--', 'translate', input_file, output_file, '-x', '10'])
        self.assertEqual(result.returncode, 0)
        result = run(['cargo', 'run', '--', 'translate', '-i', output_file, '-x', '(-10)'])
        self.assertEqual(result.returncode, 0)
        result = run(['cargo', 'run', '--', 'compare', input_file, output_file], stderr=PIPE, stdout=PIPE)
        self.assertEqual(result.returncode, 0)
        os.remove(output_file)

    def test_rotate(self):
        input_file = 'test_data/first.egsphsp1'
        output_file = 'test_data/test_rotate.egsphsp1'
        result = run(['cargo', 'run', '--', 'rotate', input_file, output_file, '-a', str(math.pi)])
        self.assertEqual(result.returncode, 0)
        result = run(['cargo', 'run', '--', 'rotate', '-i', output_file, '-a', str(math.pi)])
        self.assertEqual(result.returncode, 0)
        result = run(['cargo', 'run', '--', 'compare', input_file, output_file], stderr=PIPE, stdout=PIPE)
        self.assertEqual(result.returncode, 0)
        os.remove(output_file)

    def test_reflect(self):
        input_file = 'test_data/first.egsphsp1'
        output_file = 'test_data/test_reflect.egsphsp1'
        result = run(['cargo', 'run', '--', 'reflect', input_file, output_file, '-x', '1'])
        self.assertEqual(result.returncode, 0)
        result = run(['cargo', 'run', '--', 'reflect', '-i', output_file, '-x', '(-1)'])
        self.assertEqual(result.returncode, 0)
        result = run(['cargo', 'run', '--', 'compare', input_file, output_file], stderr=PIPE, stdout=PIPE)
        self.assertEqual(result.returncode, 0)
        os.remove(output_file)


if __name__ == '__main__':
    main()
