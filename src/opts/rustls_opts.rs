#![cfg(feature = "rustls-tls")]

use rustls::{Certificate, PrivateKey};
use rustls_pemfile::{certs, rsa_private_keys};

use std::{borrow::Cow, path::Path};

use super::PathOrBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientIdentity {
    cert_chain: PathOrBuf<'static>,
    priv_key: PathOrBuf<'static>,
}

impl ClientIdentity {
    /// Creates new identity.
    ///
    /// `cert_chain` - certificate chain (in PEM or DER)
    /// `priv_key` - private key (in DER or PEM) (it'll take the first one)
    pub fn new(cert_chain: PathOrBuf<'static>, priv_key: PathOrBuf<'static>) -> Self {
        Self {
            cert_chain,
            priv_key,
        }
    }

    /// Sets the certificate chain path (in DER or PEM).
    pub fn with_cert_chain(mut self, cert_chain: PathOrBuf<'static>) -> Self {
        self.cert_chain = cert_chain;
        self
    }

    /// Sets the private key path (in DER or PEM) (it'll take the first one).
    pub fn with_priv_key<T>(mut self, priv_key: PathOrBuf<'static>) -> Self
    where
        T: Into<Cow<'static, Path>>,
    {
        self.priv_key = priv_key;
        self
    }

    /// Returns the certificate chain.
    pub fn cert_chain(&self) -> PathOrBuf<'_> {
        self.cert_chain.borrow()
    }

    /// Returns the private key.
    pub fn priv_key(&self) -> PathOrBuf<'_> {
        self.priv_key.borrow()
    }

    pub(crate) async fn load(&self) -> crate::Result<(Vec<Certificate>, PrivateKey)> {
        let cert_data = self.cert_chain.read().await?;
        let key_data = self.priv_key.read().await?;

        let mut cert_chain = Vec::new();
        if std::str::from_utf8(&cert_data).is_err() {
            cert_chain.push(Certificate(cert_data.into_owned()));
        } else {
            for cert in certs(&mut &*cert_data)? {
                cert_chain.push(Certificate(cert));
            }
        }

        let priv_key = if std::str::from_utf8(&key_data).is_err() {
            Some(PrivateKey(key_data.into_owned()))
        } else {
            rsa_private_keys(&mut &*key_data)?
                .into_iter()
                .take(1)
                .map(PrivateKey)
                .next()
        };

        Ok((
            cert_chain,
            priv_key.ok_or_else(|| crate::Error::from(crate::DriverError::NoKeyFound))?,
        ))
    }
}
