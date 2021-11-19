{
  'targets': [
    {
      'target_name': 'winplayerbinding',
      'sources': [ 'src/main.cpp' ],
      'include_dirs': ["<!@(node -p \"require('node-addon-api').include\")" ],
	  'libraries': [ 'WindowsApp.lib' ],
      'cflags!': [ '-fno-exceptions' ],
      'cflags_cc!': [ '-fno-exceptions' ],
      'msvs_settings': {
        'VCCLCompilerTool': {
          'ExceptionHandling': 1,
          'AdditionalOptions': ['/std:c++17', '/await'],
        },
      },
    }
  ]
}
