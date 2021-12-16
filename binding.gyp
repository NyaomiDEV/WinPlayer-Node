{
	'targets': [
		{
			'target_name': 'winplayerbinding',
			'sources': [],
			"conditions": [
				["OS=='win'", {
					'include_dirs': ["<!@(node -p \"require('node-addon-api').include\")"],
					'cflags!': ['-fno-exceptions'],
					'cflags_cc!': ['-fno-exceptions'],
					'sources': [ 'src/main.cpp', 'src/wrapper.cpp', 'src/updateworker.cpp', 'src/lib/winPlayer.cpp' ],
					'libraries': [ 'WindowsApp.lib' ],
					'msvs_settings': {
						'VCCLCompilerTool': {
							'ExceptionHandling': 1,
							'AdditionalOptions': ['/std:c++17', '/await'],
						},
					},
				}]
			]
		}
	]
}
