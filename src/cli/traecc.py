#!/usr/bin/env python3
import argparse
import os
import sys
import time

if getattr(sys, 'frozen', False):
    sys.path.insert(0, os.path.join(os.path.dirname(sys.executable), '..', 'src'))
else:
    sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

from core.setup_builder import SetupBuilder


def main():
    parser = argparse.ArgumentParser(
        prog='TraeCC',
        description='Trae Installer Creator Command-Line Compiler',
        epilog='Example: traecc MyProgram.iss -O "d:\\output" -F "MyProgramSetup"'
    )
    
    parser.add_argument('script', help='Path to the .iss script file')
    parser.add_argument('-O', '--output-dir', dest='output_dir', help='Output directory')
    parser.add_argument('-F', '--output-name', dest='output_name', help='Output filename')
    parser.add_argument('-Q', '--quiet', dest='quiet', action='store_true', help='Quiet mode')
    parser.add_argument('-V', '--verbose', dest='verbose', action='store_true', help='Verbose mode')
    
    args = parser.parse_args()
    
    if not os.path.exists(args.script):
        print(f'Error: Script file "{args.script}" not found', file=sys.stderr)
        sys.exit(1)
    
    if not args.quiet:
        print(f'Trae Installer Creator Command-Line Compiler v1.0')
        print(f'Copyright (C) 2026 Trae. All rights reserved.\n')
        print(f'Compiling script: {args.script}')
    
    start_time = time.time()
    
    try:
        builder = SetupBuilder()
        builder.load_script(args.script)
        
        output_path = None
        if args.output_dir and args.output_name:
            output_path = os.path.join(args.output_dir, args.output_name + '.exe')
        elif args.output_dir:
            output_path = os.path.join(args.output_dir, os.path.basename(args.script).replace('.iss', '.exe'))
        
        result = builder.build(output_path)
        
        elapsed_time = time.time() - start_time
        
        if not args.quiet:
            print(f'Successfully compiled setup to: {result}')
            print(f'Time elapsed: {elapsed_time:.2f} seconds')
        
        sys.exit(0)
        
    except Exception as e:
        print(f'Error: {str(e)}', file=sys.stderr)
        if args.verbose:
            import traceback
            traceback.print_exc()
        sys.exit(1)


if __name__ == '__main__':
    main()