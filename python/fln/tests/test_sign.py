from fln import KeyPair, SignedClaim


def test_sign_and_verify_roundtrip():
    kp = KeyPair.generate()
    claim = SignedClaim.new(kp, b"BTC entry thesis v1")
    assert claim.verify()


def test_tampered_payload_fails_verify():
    kp = KeyPair.generate()
    claim = SignedClaim.new(kp, b"BTC entry thesis v1")
    claim.payload = b"BTC entry thesis v2"
    assert not claim.verify()


def test_keypair_from_bytes_roundtrip():
    kp = KeyPair.generate()
    secret = kp.secret_bytes()
    assert len(secret) == 32
    kp2 = KeyPair.from_bytes(secret)
    assert kp.public_bytes() == kp2.public_bytes()
