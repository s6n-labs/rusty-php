<?php

require_once __DIR__ . '/vendor/autoload.php';

use PHPJava\Core\JavaClass;
use PHPJava\Core\Stream\Reader\FileReader;

error_reporting(E_ALL & ~E_NOTICE & ~E_DEPRECATED);

JavaClass::load('HelloWorld')
    ->getInvoker()
    ->getStatic()
    ->getMethods()
    ->call(
        'main',
        ["Hello World!"]
    );
