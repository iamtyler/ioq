/****************************************************************************
*
*   net/endpoint.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

// TODO: support IPv6


/****************************************************************************
*
*   IpAddrV4
*
***/

#[derive(Copy, Clone)]
pub struct IpAddrV4 {
    octets: [u8; 4],
}

impl IpAddrV4 {
    pub fn octets (&self) -> &[u8; 4] { &self.octets }

    //=======================================================================
    pub fn new_from_octets (o1: u8, o2: u8, o3: u8, o4: u8) -> IpAddrV4 {
        IpAddrV4 { octets: [o1, o2, o3, o4] }
    }

    //=======================================================================
    pub fn new_unspecified () -> IpAddrV4 {
        IpAddrV4 { octets: [0; 4] }
    }
}


/****************************************************************************
*
*   EndpointV4
*
***/

#[derive(Copy, Clone)]
pub struct EndpointV4 {
    address: IpAddrV4,
    port: u16,
}

impl EndpointV4 {
    pub fn address (&self) -> &IpAddrV4 { &self.address }
    pub fn port (&self) -> u16 { self.port }

    //=======================================================================
    pub fn new (address: IpAddrV4, port: u16) -> EndpointV4 {
        EndpointV4 {
            address: address,
            port: port,
        }
    }
}


/****************************************************************************
*
*   Endpoint
*
***/

#[derive(Copy, Clone)]
pub enum Endpoint {
    V4(EndpointV4)
}

impl Endpoint {
    //=======================================================================
    pub fn new_v4 (address: IpAddrV4, port: u16) -> Endpoint {
        Endpoint::V4(EndpointV4::new(address, port))
    }
}

