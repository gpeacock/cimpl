#!/usr/bin/env python3
"""
Test script for secret message processor Python bindings
"""

import secret

def test_encoding():
    print("=== Testing Encoding Functions ===")
    
    # ROT13
    encoded = secret.rot13("Hello World")
    print(f"ROT13('Hello World') = '{encoded}'")
    assert encoded == "Uryyb Jbeyq"
    
    # Reverse
    reversed_text = secret.reverse("Hello")
    print(f"reverse('Hello') = '{reversed_text}'")
    assert reversed_text == "olleH"
    
    # Remove vowels
    no_vowels = secret.remove_vowels("Hello World")
    print(f"remove_vowels('Hello World') = '{no_vowels}'")
    assert no_vowels == "Hll Wrld"
    
    # Uppercase
    upper = secret.uppercase("hello")
    print(f"uppercase('hello') = '{upper}'")
    assert upper == "HELLO"
    
    print("✓ All encoding tests passed\n")

def test_hex():
    print("=== Testing Hex Encoding/Decoding ===")
    
    # Encode to hex
    hex_str = secret.to_hex("Hello")
    print(f"to_hex('Hello') = '{hex_str}'")
    assert hex_str == "48656c6c6f"
    
    # Decode from hex
    decoded = secret.from_hex(hex_str)
    print(f"from_hex('{hex_str}') = '{decoded}'")
    assert decoded == "Hello"
    
    # Test invalid hex
    try:
        secret.from_hex("invalid")
        assert False, "Should have raised error"
    except secret.SecretError as e:
        print(f"✓ Correctly raised error for invalid hex: {e}")
        assert e.code == secret.SECRET_ERROR_INVALID_HEX
    
    print("✓ All hex tests passed\n")

def test_validation():
    print("=== Testing Validation Functions ===")
    
    # Valid length
    assert secret.validate_length("hello", 1, 10)
    print("✓ validate_length('hello', 1, 10) = True")
    
    # Too short
    try:
        secret.validate_length("hi", 5, 10)
        assert False, "Should have raised error"
    except secret.SecretError as e:
        print(f"✓ Correctly raised error for too short: {e}")
        assert e.code == secret.SECRET_ERROR_TOO_SHORT
    
    # Too long
    try:
        secret.validate_length("hello world", 1, 5)
        assert False, "Should have raised error"
    except secret.SecretError as e:
        print(f"✓ Correctly raised error for too long: {e}")
        assert e.code == secret.SECRET_ERROR_TOO_LONG
    
    # ASCII check
    assert secret.is_ascii("hello")
    assert not secret.is_ascii("hello 世界")
    print("✓ is_ascii tests passed")
    
    # Hex validation
    assert secret.is_valid_hex("48656c6c6f")
    assert not secret.is_valid_hex("xyz")
    print("✓ is_valid_hex tests passed")
    
    print("✓ All validation tests passed\n")

def test_counting():
    print("=== Testing Counting Functions ===")
    
    text = "Hello World"
    
    chars = secret.count_chars(text)
    print(f"count_chars('{text}') = {chars}")
    assert chars == 11
    
    vowels = secret.count_vowels(text)
    print(f"count_vowels('{text}') = {vowels}")
    assert vowels == 3
    
    consonants = secret.count_consonants(text)
    print(f"count_consonants('{text}') = {consonants}")
    assert consonants == 7
    
    words = secret.count_words(text)
    print(f"count_words('{text}') = {words}")
    assert words == 2
    
    print("✓ All counting tests passed\n")

def main():
    print("Testing Secret Message Processor Python Bindings")
    print("=" * 50)
    print()
    
    test_encoding()
    test_hex()
    test_validation()
    test_counting()
    
    print("=" * 50)
    print("✅ All tests passed!")

if __name__ == '__main__':
    main()
