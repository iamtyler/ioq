/****************************************************************************
*
*   net/addr.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

use std::fmt;
use std::option::IntoIter;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6};

use error::Error;


/****************************************************************************
*
*   Traits
*
***/

pub trait ToSocketAddrs {
    type Iter: Iterator<Item=SocketAddr>;
    fn to_socket_addrs(&self) -> Result<Self::Iter, Error>;
}


/****************************************************************************
*
*   AddrFamily
*
***/

pub enum AddrFamily {
    Ipv4,
    Ipv6,
}


/****************************************************************************
*
*   IpAddr
*
***/

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, PartialOrd, Ord)]
pub enum IpAddr {
    V4(Ipv4Addr),
    V6(Ipv6Addr),
}

impl IpAddr {
    //=======================================================================
    pub fn family (&self) -> AddrFamily {
        match *self {
            IpAddr::V4(..) => AddrFamily::Ipv4,
            IpAddr::V6(..) => AddrFamily::Ipv6,
        }
    }
}

impl fmt::Display for IpAddr {
    //=======================================================================
    fn fmt (&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IpAddr::V4(ref a) => a.fmt(fmt),
            IpAddr::V6(ref a) => a.fmt(fmt),
        }
    }
}


/****************************************************************************
*
*   SocketAddr
*
***/

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum SocketAddr {
    V4(SocketAddrV4),
    V6(SocketAddrV6),
}

impl SocketAddr {
    //=======================================================================
    pub fn new (ip: IpAddr, port: u16) -> SocketAddr {
        match ip {
            IpAddr::V4(a) => SocketAddr::V4(SocketAddrV4::new(a, port)),
            IpAddr::V6(a) => SocketAddr::V6(SocketAddrV6::new(a, port, 0, 0)),
        }
    }

    //=======================================================================
    pub fn ip (&self) -> IpAddr {
        match *self {
            SocketAddr::V4(ref a) => IpAddr::V4(*a.ip()),
            SocketAddr::V6(ref a) => IpAddr::V6(*a.ip()),
        }
    }

    //=======================================================================
    pub fn port (&self) -> u16 {
        match *self {
            SocketAddr::V4(ref a) => a.port(),
            SocketAddr::V6(ref a) => a.port(),
        }
    }

    //=======================================================================
    pub fn family (&self) -> AddrFamily {
        match *self {
            SocketAddr::V4(..) => AddrFamily::Ipv4,
            SocketAddr::V6(..) => AddrFamily::Ipv6,
        }
    }
}

impl fmt::Display for SocketAddr {
    //=======================================================================
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SocketAddr::V4(ref a) => a.fmt(f),
            SocketAddr::V6(ref a) => a.fmt(f),
        }
    }
}

impl ToSocketAddrs for SocketAddr {
    //=======================================================================
    type Iter = IntoIter<SocketAddr>;
    fn to_socket_addrs (&self) -> Result<IntoIter<SocketAddr>, Error> {
        Ok(Some(*self).into_iter())
    }
}